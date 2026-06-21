use libs::{ fetchers::calls::fetch_calls_table, tables::IxTable };
use book::{
    table_utils::val,
    err_utils::ErrStr,
    date_utils::parse_date,
    num_utils::parse_num,
    currency::usd::{ USD, mk_usd },
    utils::{ get_env, get_args },
    parse_utils::{ parse_id, parse_usd }
};

// ====================================================
//----- pub fn header ---------------------------------
// ====================================================
pub fn header() -> String {
    let line1 = "date,pivot,close,tx_id,from,from_quote";
    let line2 = "to,to_quote,trade,vol,gain_10_percent";
    let line3 = "new_to_actual,gain,gain_total_usd,roi,apr";
    format!("{line1},{line2},{line3}")
}

// ====================================================
//----- pub fn parse_row ------------------------------
// ====================================================
pub fn parse_row(table: &IxTable, ix: usize, tx_id: &str, new_to_actual: &str) -> ErrStr<String> {
    let col = |name: &str| -> ErrStr<String> {
        let v = val(&table, &ix, &name.to_string()).unwrap_or_default();
        if v.is_empty() {
            Err(format!("missing data or empty column '{name}'"))
        } else {
            Ok(v)
        }
    };    
    let col_num = |name: &str| -> ErrStr<f32> {
        let raw = col(name)?;
        parse_num(raw.trim())
    };
    let col_opt = |name: &str| -> ErrStr<USD> {
        let raw = col(name)?;
        parse_usd(raw.trim())
    };
    //----- truth values -------------------------------
    let date   = parse_date(&col("close_date")?)?;
    let opened = parse_date(&col("opened")?)?;
    let pivot     = col("ids")?;
    let close     = col("close_id")?;
    let from      = col("pivot_token")?;
    let to        = col("from")?;
    let trade        = col_num("pivot_amount")?;
    let amount1      = col_num("amount1")?;
    let virtual_     = col_num("virtual")?;
    let actual       = parse_num(new_to_actual)?;
    let from_quote   = col_opt("pivot_close_price")?;
    let to_quote     = col_opt("proposed_close_price")?;
    //----- formulas for the correct headers -----------
    let sum_amt_virt    = amount1 + virtual_;
    let vol             = mk_usd(trade * from_quote.amount());
    let gain_10_percent = sum_amt_virt * 1.1;
    let gain            = actual - sum_amt_virt;
    let gain_total_usd  = mk_usd(gain * to_quote.amount());
    let roi_val         = if sum_amt_virt != 0.0 { gain / sum_amt_virt } else { 0.0 };
    let days            = (date - opened).num_days();
    if days < 0 {
        return Err(format!("opened date '{opened}' is after close date '{date}', cannot compute APR"));
    }
    let days_f32        = days as f32;
    let apr_val         = if days > 0 { (roi_val * 365.0) / days_f32 } else { 0.0 };
    //----- formatting the actual output ---------------
    let line1 = format!("{date},{pivot},{close},{tx_id},{from},{from_quote}");
    let line2 = format!("{to},{to_quote},{trade},{vol:.4},{gain_10_percent:.4}");
    let line3 = format!("{actual},{gain:.4},{gain_total_usd:.2},{:.2}%,{:.2}%",
                        roi_val * 100.0, apr_val * 100.0);
    Ok(format!("{line1},{line2},{line3}"))
}

//----- fn pool_path ----------------------------------

pub fn pool_path(close_dir: &str, table: &IxTable, ix: usize) -> ErrStr<String> {
    let pivot_token = val(table, &ix, &"pivot_token".to_string()).unwrap_or_default();
    let from        = val(table, &ix, &"from".to_string()).unwrap_or_default();
    if pivot_token.is_empty() || from.is_empty() {
        return Err("missing pivot_token or from column/data".to_string());
    }
    let a = pivot_token.to_lowercase();
    let b = from.to_lowercase();
    let (left, right) = if a <= b { (&a, &b) } else { (&b, &a) };
    Ok(format!("{close_dir}/{left}-{right}.tsv"))
}

// =====================================================
//----- UNIT TESTS -------------------------------------
// =====================================================

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod test_functions {
    use super::*;
    use book::{ parse_utils::parse_str, string_utils::s, table_utils::ingest };

    pub fn make_table(raw: &str) -> ErrStr<IxTable> {
        let lines: Vec<String> = raw.lines().map(s).collect();
        ingest(parse_id, parse_str, parse_str, &lines, ",")
    }
}

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod tests {
    use super::*;
    use super::test_functions::make_table;
    use book::{
        date_utils::today,
        string_utils::s,
        parse_utils::{  parse_id, parse_str },
        table_utils::{ ingest, row }
    };

    fn mock_dusk_output() -> String {
        s("ix,pool,ids\n1,BTC+UNDEAD,20\n2,ETH+UNDEAD,15\n3,SOL+UNDEAD,10")
    }

    fn mock_trade_row() -> String {
        let dt = format!("{}", today());
        format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price
1,{dt},{dt},20,99,BTC,UNDEAD,500,100,50,2.00,2.00"
        )
    }

    fn mock_losing_row() -> String {
        let dt = format!("{}", today());
        format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price
1,{dt},{dt},20,99,BTC,UNDEAD,0,100,50,0.00,0.00"
        )
    }

    fn mock_missing_dates() -> String { s("ix,amount1,virtual\n1,100,50") }

    #[test]
    fn test_ix_in_bounds() -> ErrStr<()> {
        let raw_data = mock_dusk_output();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines()
            .map(s).collect(),",")?;
        assert!(row(&table, &1).is_some());
        assert!(row(&table, &3).is_some());
        Ok(())
    }

    #[test]
    fn test_ix_out_of_bounds() -> ErrStr<()> {
        let raw_data = mock_dusk_output();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines()
            .map(s).collect(),",")?;
        assert!(row(&table, &99).is_none());
        Ok(())
    }

    #[test]
    fn test_parse_row_returns_ok() -> ErrStr<()> {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines()
            .map(s).collect(), ",")?;
        assert!(parse_row(&table, 1, "tx1", "160.0").is_ok());
        Ok(())
    }

    #[test]
    fn test_parse_row_err_on_missing_dates() -> ErrStr<()> {
        let raw_data = mock_missing_dates();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines()
            .map(s).collect(), ",")?;
        assert!(parse_row(&table, 1, "tx1", "160.0").is_err());
        Ok(())
    }

    #[test]
    fn test_parse_row_field_count_matches_header() -> ErrStr<()> {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines()
            .map(s).collect(), ",")?;
        let row_str = parse_row(&table, 1, "tx1", "160.0").unwrap();
        assert_eq!(header().split(',').count(), row_str.split(',').count());
        Ok(())
    }

    #[test]
    fn test_parse_row_passthrough_tx_id() -> ErrStr<()> {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines()
            .map(s).collect(), ",")?;
        let row_str = parse_row(&table, 1, "my-tx-abc", "160.0").unwrap();
        assert!(row_str.contains("my-tx-abc"));
        Ok(())
    }

    #[test]
    fn test_parse_row_vol_calculation() -> ErrStr<()> {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines()
            .map(s).collect(), ",")?;
        let row_str = parse_row(&table, 1, "tx1", "160.0").unwrap();
        assert!(row_str.contains("$1000.00"), "expected $1000.00 in: {row_str}");
        Ok(())
    }

    #[test]
    fn test_parse_row_gain_10_percent() -> ErrStr<()> {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines()
            .map(s).collect(), ",")?;
        let row_str = parse_row(&table, 1, "tx1", "160.0").unwrap();
        assert!(row_str.contains("165.0000"), "expected 165.0000 in: {row_str}");
        Ok(())
    }

    #[test]
    fn test_parse_row_negative_gain() -> ErrStr<()> {
        let raw_data = mock_losing_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines()
            .map(s).collect(), ",")?;
        let row_str = parse_row(&table, 1, "tx1", "100.0").unwrap();
        let gain: f32 = row_str.split(',').nth(12).unwrap_or("0").parse().unwrap_or(0.0);
        assert!(gain < 0.0, "expected negative gain, got {gain}");
        Ok(())
    }

    #[test]
    fn test_parse_row_no_nan_or_inf() -> ErrStr<()> {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines()
            .map(s).collect(), ",")?;
        let row_str = parse_row(&table, 1, "tx1", "160.0").unwrap();
        assert!(!row_str.contains("NaN") && !row_str.contains("inf"),
            "NaN or inf in output: {row_str}");
        Ok(())
    }

    #[test]
    fn test_parse_row_comma_formatted_new_to_actual() -> ErrStr<()> {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines().map(s).collect(), ",")?;
        let row_str = parse_row(&table, 1, "tx1", "883,619.4538")?;
        assert_eq!(header().split(',').count(), row_str.split(',').count(),
            "comma in new_to_actual broke field count: {row_str}");
        assert!(!row_str.contains("883,619"),
            "raw comma-formatted value leaked into CSV output: {row_str}");
        Ok(())
    }

    #[test]
    fn test_parse_row_currency_fields_have_dollar_prefix() -> ErrStr<()> {
        let dt = format!("{}", today());
        let raw = format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},20,99,BTC,UNDEAD,500,100,50,2.00,3.00"
        );
        let table = ingest(
            parse_id, parse_str, parse_str, &raw.lines().map(s).collect(), ",",)?;
        let row_str = parse_row(&table, 1, "tx1", "160.0").unwrap();
        let fields: Vec<&str> = row_str.split(',').collect();
        assert!(fields[5].starts_with('$'),  "from_quote missing $: '{}'", fields[5]);
        assert!(fields[9].starts_with('$'),  "vol missing $: '{}'", fields[9]);
        assert!(fields[13].starts_with('$'), "gain_total_usd missing $: '{}'", fields[13]);
        Ok(())
    }

    #[test]
    fn test_truth_values_populated() -> ErrStr<()> {
        let dt = format!("{}", today());
        let raw = format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},20,99,BTC,UNDEAD,500,100,50,2.00,2.00"
        );
        let table = ingest(parse_id, parse_str, parse_str, &raw.lines()
            .map(s).collect(), ",")?;
        let result = parse_row(&table, 1, "tx1", "160.0");
        assert!(result.is_ok(), "parse_row failed: {:?}", result);
        let row_str = result.unwrap();
        let fields: Vec<&str> = row_str.split(',').collect();
        assert_eq!(fields.len(), 16);
        for (i, f) in fields.iter().enumerate() {
            assert!(!f.is_empty(), "field at index {i} is empty: full row: {row_str}");
        }
        Ok(())
    }
    
    #[test]
    fn test_parse_row_inverted_dates_returns_error() -> ErrStr<()> {
        let close  = today();
        let opened = close + chrono::Duration::days(365);
        let raw = format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{close},{opened},20,99,BTC,UNDEAD,500,100,50,2.00,2.00"
        );
        let table = ingest(parse_id, parse_str, parse_str,
            &raw.lines().map(s).collect(), ",")?;
        let result = parse_row(&table, 1, "tx1", "160.0");
        assert!(result.is_err(), "expected Err for inverted dates, got Ok");
        let msg = result.unwrap_err();
        assert!(msg.contains("cannot compute APR"),
            "expected APR error message, got: {msg}");
        Ok(())
    }

    #[test]
    fn test_parse_row_err_on_empty_to_quote() -> ErrStr<()> {
        let dt = format!("{}", today());
        let raw = format!(
            "ix,close_date,opened,pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},500,100,50,2.00,"
        );
        let table = ingest(parse_id, parse_str, parse_str,
            &raw.lines().map(s).collect(), ",")?;
        let result = parse_row(&table, 1, "tx1", "160.0");
        assert!(result.is_err(),
                "expected Err when proposed_close_price is empty, got Ok");
        Ok(())
    }

    #[test]
    fn test_pool_path_format() -> ErrStr<()> {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str,
            &raw_data.lines().map(s).collect(), ",")?;
        let path = pool_path("close_pivots", &table, 1).unwrap();
        assert_eq!(path, "close_pivots/btc-undead.tsv");
        Ok(())
    }

    #[test]
    fn test_pool_path_eth_undead() -> ErrStr<()> {
        let raw = format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},20,99,ETH,UNDEAD,0,0,0,0.00,0.00",
            dt = today()
        );
        let table = ingest(parse_id, parse_str, parse_str,
            &raw.lines().map(s).collect(), ",")?;
        assert_eq!(pool_path("cpp", &table, 1).unwrap(), "cpp/eth-undead.tsv");
        Ok(())
    }

    #[test]
    fn test_pool_path_btc_eth_alphabetical_order() -> ErrStr<()> {
        let raw = format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},20,99,ETH,BTC,0,0,0,0.00,0.00",
            dt = today()
        );
        let table = ingest(parse_id, parse_str, parse_str,
            &raw.lines().map(s).collect(), ",")?;
        assert_eq!(pool_path("geophf_is_grate", &table, 1).unwrap(),
            "geophf_is_grate/btc-eth.tsv");
        Ok(())
    }

    #[test]
    fn test_pool_path_undead_usdc_alphabetical_order() -> ErrStr<()> {
        let raw = format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},20,99,USDC,UNDEAD,0,0,0,0.00,0.00",
            dt = today()
        );
        let table = ingest(parse_id, parse_str, parse_str,
            &raw.lines().map(s).collect(), ",")?;
        assert_eq!(pool_path("dpcr", &table, 1).unwrap(),
            "dpcr/undead-usdc.tsv");
        Ok(())
    }

    #[test]
    fn missing_table_close_date_header() -> ErrStr<()> {
        let table = make_table("ix,pool,ids\n1,BTC,20")?;
        let row = parse_row(&table, 1, "tx_id", "1.0");
        assert!(row.is_err());
        Ok(())  
    }

}

//----- fn runoff_with_args -----------------------------------

fn usage() -> ErrStr<()> {
   eprintln!("Error: not enough arguments.");
   eprintln!("Usage: `wyrd` <auth> <path> <ix> <tx_id> <new_to_actual>");
   eprintln!("Example: wyrd PIVOT data/pivots/close/raw 5 asdf 1250.75");
   let arguments = "<close_pivot_dir> <close_ix> <tx_id> <actual amount>";
   Err(format!("wyrd missing arguments <auth> {arguments}"))
}

pub async fn runoff_with_args() -> ErrStr<()> {
    let args = get_args();
    if args.len() < 5 {
       usage()
    } else {
       runoff_continuation(&args).await
    }
}

async fn runoff_continuation(args: &[String]) -> ErrStr<()> {
    let protocol = &args[0];
    let protocol_up = protocol.to_uppercase();
    let close_dir = &args[1];
    let ix       = parse_id(&args[2])?;
    let root_url = get_env(&format!("{protocol_up}_URL"))?;
    let calls = fetch_calls_table(&root_url).await?;
    println!("{}", header());
    println!("{}", parse_row(&calls, ix, &args[3], &args[4])?);
    println!("{}", pool_path(&close_dir, &calls, ix)?);
    Ok(())
}

// =====================================================
//----- FUNCTIONAL TESTS -------------------------------
// =====================================================

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
    use super::*;
    use super::test_functions::make_table;
    use paste::paste;
    use book::{ date_utils::today, utils::resolve, create_testing, compose };

    fn now() -> String { format!("{}", today()) }

    create_testing!("quiz08::b_wyrd", "", true);

    run!("parse_row", {
        let raw_data = 
        "ix,pool,open_pivots,last_pivot_on_dt,opened,ids,close_id,close_date,from,from_blockchain,amount1,virtual,quote1,val1,gain_10_percent,pivot_token,pivot_blockchain,pivot_close_price,pivot_amount,proposed_token,proposed_blockchain,proposed_close_price,proposed_amount,roi,apr
1,BTC+ETH,27,2026-05-07,2025-11-05,46,14,2026-05-19,ETH,Avalanche,0,0.1498,$3340.95,$500.47,0.16478,BTC,Avalanche,$76815.00,0.00467,ETH,Avalanche,$2116.94,0.169455,13.12%,24.56%";
        let table = make_table(raw_data)?;
        let row = parse_row(&table, 1, "asdf", "0.17")?;
        println!("result: {row} ");
    });

    fn apr_safety(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},20,99,BTC,UNDEAD,0,0,1,0.00,0.00"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "1.0"))?;
        if row.contains("NaN") || row.contains("inf") { Err(format!("apr_safety: NaN or inf in: {row}")) } else{
        Ok(row)}
    }
    run_with!("apr_safety", now(), compose!(resolve)(apr_safety));

    fn whale(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},20,99,BTC,UNDEAD,150000000,0,0,1.50,1.50"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "1.0"));
        row
    }
    run_with!("whale", now(), compose!(resolve)(whale));

    fn roi_zero_div(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},20,99,BTC,UNDEAD,0,0,0,0.00,0.00"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "0.0"))?;
        if row.contains("NaN") || row.contains("inf") { Err(format!("roi_zero_div: NaN or inf in: {row}")) } else{
        Ok(row)}
    }
    run_with!("roi_zero_div", now(), compose!(resolve)(roi_zero_div));

    fn column_count(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},20,99,BTC,UNDEAD,0,100,50,0.00,0.00"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "120.0"))?;
        let (h, r) = (header().split(',').count(), row.split(',').count());
        if h != r { Err(format!("column_count: header={h} row={r}")) } else{
        Ok(row)}
    }
    run_with!("column_count", now(), compose!(resolve)(column_count));

    fn currency_format(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},20,99,BTC,UNDEAD,100000,0,0,$1.50,$1.50"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "1.0"))?;
        if !row.contains("150000.00") { return Err(format!("currency_format: expected 150000.00 in: {row}")); }
        Ok(row)
    }
    run_with!("currency_format", now(), compose!(resolve)(currency_format));

    fn undead_zero_precision(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},20,99,BTC,UNDEAD,1000,500,100,0.0025,0.0025"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "650.0"))?;
        let gain_usd: f32 = row.split(',').nth(13).unwrap_or("$0").replace('$', "").parse().unwrap_or(0.0);
        if gain_usd == 0.0 { return Err("undead_zero_precision: gain_total_usd is 0, to_quote not rescued".to_string()); }
        Ok(row)
    }

    run_with!("undead_zero_precision", now(), compose!(resolve)(undead_zero_precision));
}


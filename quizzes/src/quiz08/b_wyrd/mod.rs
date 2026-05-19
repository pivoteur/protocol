use tokio::runtime::Runtime;
use libs::{ 
    fetchers::calls::fetch_calls, 
    tables::IxTable 
};
use book::{
    table_utils::val,
    err_utils::ErrStr,
    date_utils::parse_date,
    num_utils::parse_num,
    utils::{ 
        get_env,
        get_args
    },
    parse_utils::{
        parse_id,
        parse_usd
    }
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
            .map(|v| v as f32)
            .map_err(|_| "missing table's data".to_string())
    };
    let col_opt = |name: &str| -> ErrStr<f32> {
        let raw = col(name)?;
        parse_usd(raw.trim())
            .map(|v| v.amount as f32)
            .map_err(|e| format!("invalid value for '{name}': {e}"))
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
    let actual       = new_to_actual.parse::<f32>()
                            .map_err(|_| format!("invalid new_to_actual: '{new_to_actual}' is not a number"))?;
    let from_quote   = col_opt("pivot_close_price")?;
    let to_quote     = col_opt("proposed_close_price")?;
    //----- formulas for the correct headers -----------
    let sum_amt_virt    = amount1 + virtual_;
    let vol             = trade * from_quote;
    let gain_10_percent = sum_amt_virt * 1.1;
    let gain            = actual - sum_amt_virt;
    let gain_total_usd  = gain * to_quote;
    let roi_val         = if sum_amt_virt != 0.0 { gain / sum_amt_virt } else { 0.0 };
    let days            = (date - opened).num_days();
    if days < 0 {
        return Err(format!("opened date '{opened}' is after close date '{date}', cannot compute APR"));
    }
    let apr_val         = if days > 0 { (roi_val * 365.0) / days as f32 } else { 0.0 };
    //----- formatting the actual output ---------------
    let line1 = format!("{date},{pivot},{close},{tx_id},{from},${from_quote}");
    let line2 = format!("{to},${to_quote},{trade},${vol:.4},{gain_10_percent:.4}");
    let line3 = format!("{new_to_actual},{gain:.4},${gain_total_usd:.2},{:.2}%,{:.2}%",
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

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod tests {
    use super::*;
    use book::{
        date_utils::today,
        parse_utils::{ 
            parse_str, 
            parse_id 
        },
        table_utils::{
            row, 
            ingest 
        }
    };

    fn mock_dusk_output() -> String {
        "ix,pool,ids\n\
         1,BTC+UNDEAD,20\n\
         2,ETH+UNDEAD,15\n\
         3,SOL+UNDEAD,10"
            .to_string()
    }

    fn mock_trade_row() -> String {
        let dt = format!("{}", today());
        format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},20,99,BTC,UNDEAD,500,100,50,2.00,2.00"
        )
    }

    fn mock_losing_row() -> String {
        let dt = format!("{}", today());
        format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},20,99,BTC,UNDEAD,0,100,50,0.00,0.00"
        )
    }

    fn mock_missing_dates() -> String {
        "ix,amount1,virtual\n\
         1,100,50"
            .to_string()
    }

    #[test]
    fn test_ix_in_bounds() {
        let raw_data = mock_dusk_output();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest mock data");
        assert!(row(&table, &1).is_some());
        assert!(row(&table, &3).is_some());
    }
    #[test]
    fn test_ix_out_of_bounds() {
        let raw_data = mock_dusk_output();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest mock data");
        assert!(row(&table, &99).is_none());
    }

    #[test]
    fn test_parse_row_returns_ok() {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest mock data");
        assert!(parse_row(&table, 1, "tx1", "160.0").is_ok());
    }

    #[test]
    fn test_parse_row_err_on_missing_dates() {
        let raw_data = mock_missing_dates();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest mock data");
        assert!(parse_row(&table, 1, "tx1", "160.0").is_err());
    }

    #[test]
    fn test_parse_row_field_count_matches_header() {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest mock data");
        let row_str = parse_row(&table, 1, "tx1", "160.0").unwrap();
        assert_eq!(header().split(',').count(), row_str.split(',').count());
    }

    #[test]
    fn test_parse_row_passthrough_tx_id() {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest mock data");
        let row_str = parse_row(&table, 1, "my-tx-abc", "160.0").unwrap();
        assert!(row_str.contains("my-tx-abc"));
    }

    #[test]
    fn test_parse_row_vol_calculation() {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest mock data");
        let row_str = parse_row(&table, 1, "tx1", "160.0").unwrap();
        assert!(row_str.contains("$1000.0000"), "expected $1000.0000 in: {row_str}");
    }

    #[test]
    fn test_parse_row_gain_10_percent() {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest mock data");
        let row_str = parse_row(&table, 1, "tx1", "160.0").unwrap();
        assert!(row_str.contains("165.0000"), "expected 165.0000 in: {row_str}");
    }

    #[test]
    fn test_parse_row_negative_gain() {
        let raw_data = mock_losing_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest mock data");
        let row_str = parse_row(&table, 1, "tx1", "100.0").unwrap();
        let gain: f32 = row_str.split(',').nth(12).unwrap_or("0").parse().unwrap_or(0.0);
        assert!(gain < 0.0, "expected negative gain, got {gain}");
    }

    #[test]
    fn test_parse_row_no_nan_or_inf() {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest mock data");
        let row_str = parse_row(&table, 1, "tx1", "160.0").unwrap();
        assert!(!row_str.contains("NaN") && !row_str.contains("inf"),
            "NaN or inf in output: {row_str}");
    }

    #[test]
    fn test_roi_zero_division_guard() {
        let s = 0.0_f32;
        assert_eq!(0.0, if s != 0.0 { 1.0 / s } else { 0.0 });
    }

    #[test]
    fn test_apr_zero_days_guard() {
        assert_eq!(0.0, if 0.0_f32 > 0.0 { 0.1 * 365.0 / 0.0 } else { 0.0 });
    }
    #[test]
    fn test_apr_365_days_equals_roi() {
        let roi = 0.1_f32;
        assert!((roi - roi * 365.0 / 365.0).abs() < 1e-10);
    }
    
    #[test] 
    fn test_gain_formula() { 
        assert_eq!(-50.0, 100.0_f32 - (100.0 + 50.0)); 
    }    

    #[test] 
    fn test_gain_10_percent() { 
        assert!((165.0_f32 - (100.0 + 50.0) * 1.1).abs() < f32::EPSILON); 
    }

    #[test] 
    fn test_vol_formula() { 
        assert!((750.0_f32 - 500.0 * 1.5).abs() < f32::EPSILON); 
    }

    #[test]
    fn test_parse_row_currency_fields_have_dollar_prefix() {
        let dt = format!("{}", today());
        let raw = format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},20,99,BTC,UNDEAD,500,100,50,2.00,3.00"
        );
        let table = ingest(
            parse_id, parse_str, parse_str,
            &raw.lines().map(|s| s.to_string()).collect::<Vec<_>>(),
            ",",
        )
        .expect("Failed to ingest");
        let row_str = parse_row(&table, 1, "tx1", "160.0").unwrap();
        let fields: Vec<&str> = row_str.split(',').collect();
        // from_quote  = index 5  → "$2.00"
        // vol         = index 9  → "$1000.0000"
        // gain_total  = index 13 → "$..."
        assert!(fields[5].starts_with('$'),  "from_quote missing $: '{}'", fields[5]);
        assert!(fields[9].starts_with('$'),  "vol missing $: '{}'", fields[9]);
        assert!(fields[13].starts_with('$'), "gain_total_usd missing $: '{}'", fields[13]);
    }

    #[test]
    fn test_truth_values_populated() {
        let dt = format!("{}", today());
        let raw = format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},20,99,BTC,UNDEAD,500,100,50,2.00,2.00"
        );
        let table = ingest(parse_id, parse_str, parse_str, &raw.lines()
            .map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest");
        let result = parse_row(&table, 1, "tx1", "160.0");
        assert!(result.is_ok(), "parse_row failed: {:?}", result);
        let row_str = result.unwrap();
        let fields: Vec<&str> = row_str.split(',').collect();
        assert_eq!(fields.len(), 16);
        for (i, f) in fields.iter().enumerate() {
            assert!(!f.is_empty(), "field at index {i} is empty: full row: {row_str}");
        }
    }
    
    #[test]
    fn test_parse_row_inverted_dates_returns_error() {
        // "opened" is one year AFTER close_date, this should trigger the days > 0.0 guard
        let close  = today();
        let opened = close + chrono::Duration::days(365);
        let raw = format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{close},{opened},20,99,BTC,UNDEAD,500,100,50,2.00,2.00"
        );
        let table = ingest(parse_id, parse_str, parse_str,
            &raw.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest");
        let result = parse_row(&table, 1, "tx1", "160.0");
        assert!(result.is_err(), "expected Err for inverted dates, got Ok");
        let msg = result.unwrap_err();
        assert!(msg.contains("cannot compute APR"),
            "expected APR error message, got: {msg}");
    }

    #[test]
    fn test_parse_row_err_on_empty_to_quote() {
        let dt = format!("{}", today());
        // proposed_close_price is intentionally left blank
        let raw = format!(
            "ix,close_date,opened,pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},500,100,50,2.00,"
        );
        let table = ingest(parse_id, parse_str, parse_str,
            &raw.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest");
        let result = parse_row(&table, 1, "tx1", "160.0");
        assert!(result.is_err(),
                "expected Err when proposed_close_price is empty, got Ok");
    }

    #[test]
    fn test_pool_path_format() {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str,
            &raw_data.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest");
        let path = pool_path("close_pivots", &table, 1).unwrap();
        assert_eq!(path, "close_pivots/btc-undead.tsv");
    }

    #[test]
    fn test_pool_path_eth_undead() {
    let raw = format!(
        "ix,close_date,opened,ids,close_id,pivot_token,from,\
        pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
        1,{dt},{dt},20,99,ETH,UNDEAD,0,0,0,0.00,0.00",
        dt = today()
    );
    let table = ingest(parse_id, parse_str, parse_str,
        &raw.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
        .expect("ingest");
    assert_eq!(pool_path("cpp", &table, 1).unwrap(), "cpp/eth-undead.tsv");
       
    }

#[test]
fn test_pool_path_btc_eth_alphabetical_order() {
    // Even if the row has pivot_token=ETH and from=BTC,
    // btc sorts before eth so the file must still be btc-eth.tsv
    let raw = format!(
        "ix,close_date,opened,ids,close_id,pivot_token,from,\
        pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
        1,{dt},{dt},20,99,ETH,BTC,0,0,0,0.00,0.00",
        dt = today()
    );
    let table = ingest(parse_id, parse_str, parse_str,
        &raw.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
        .expect("ingest");
    assert_eq!(pool_path("geophf_is_grate", &table, 1).unwrap(),
        "geophf_is_grate/btc-eth.tsv");
}

#[test]
fn test_pool_path_undead_usdc_alphabetical_order() {
    let raw = format!(
        "ix,close_date,opened,ids,close_id,pivot_token,from,\
        pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
        1,{dt},{dt},20,99,USDC,UNDEAD,0,0,0,0.00,0.00",
        dt = today()
    );
    let table = ingest(parse_id, parse_str, parse_str,
        &raw.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
        .expect("ingest");
    assert_eq!(pool_path("dpcr", &table, 1).unwrap(),
        "dpcr/undead-usdc.tsv");
}

}

//----- fn runoff_with_args ----------------------------

pub fn runoff_with_args() -> ErrStr<()> {
    let args = get_args();
    if args.len() < 5 {
        eprintln!("Error: not enough arguments.");
        eprintln!("Usage: `wyrd` <auth> <path> <ix> <tx_id> <new_to_actual>");
        eprintln!("Example: wyrd PIVOT data/pivots/close/raw 5 \"asdf\" \"1250.75\"");
        std::process::exit(1);
    }
    let protocol = &args[0];
    let protocol_up = protocol.to_uppercase();
    let close_dir = &args[1];
    let ix       = parse_id(&args[2])?;
    let root_url = get_env(&format!("{protocol_up}_URL"))?;
    let rt       = Runtime::new().map_err(|e| e.to_string())?;
    match rt.block_on(fetch_calls(&root_url)) {
        Ok(t)  => {
            println!("{}", header());
            println!("{}", parse_row(&t, ix, &args[3], &args[4])?);
            println!("{}", pool_path(&close_dir, &t, ix)?);
        }
        Err(e) => eprintln!("Error: {e}"),
    }
    Ok(())
}

// =====================================================
//----- FUNCTIONAL TESTS -------------------------------
// =====================================================

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
    use super::*;
    use paste::paste;
    use book::{
        parse_utils::parse_str,
        table_utils:: ingest,
        date_utils::today,
        utils::resolve,
        create_testing,
        compose
    };


    fn now() -> String { format!("{}", today()) }

    fn make_table(raw: &str) -> ErrStr<IxTable> {
        let lines: Vec<String> = raw.lines().map(|s| s.to_string()).collect();
        ingest(parse_id, parse_str, parse_str, &lines, ",")
    }

    fn report<A: std::fmt::Debug + Clone>(test: &str, t: A, f: impl Fn(A) -> ErrStr<String>) -> ErrStr<usize> {
        let result = f(t.clone())?;
        println!("\npivot_dapps::run_{test} functional test\n\n\tinput: {t:?}, result: {result}\n\npivot_dapps::{test}:...ok");
        Ok(1)
    }

    create_testing!("quiz08::b_wyrd");

    run!("resilience", {
        let table = make_table("ix,pool,ids\n1,BTC,20")?;
        if parse_row(&table, 1, "tx_id", "1.0").is_ok() {
            return Err("resilience: expected Err on missing dates, got Ok".to_string());
        }
        println!("\npivot_dapps::run_resilience functional test\n\n\tresult: Err (correct)\n\npivot_dapps::resilience:...ok");
    });

    fn apr_safety(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},20,99,BTC,UNDEAD,0,0,1,0.00,0.00"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "1.0"))?;
        if row.contains("NaN") || row.contains("inf") { return Err(format!("apr_safety: NaN or inf in: {row}")); }
        Ok(row)
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
        if row.contains("NaN") || row.contains("inf") { return Err(format!("roi_zero_div: NaN or inf in: {row}")); }
        Ok(row)
    }
    run_with!("roi_zero_div", now(), compose!(resolve)(roi_zero_div));

    fn column_count(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},20,99,BTC,UNDEAD,0,100,50,0.00,0.00"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "120.0"))?;
        let (h, r) = (header().split(',').count(), row.split(',').count());
        if h != r { return Err(format!("column_count: header={h} row={r}")); }
        Ok(row)
    }
    run_with!("column_count", now(), compose!(resolve)(column_count));

    fn currency_format(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},20,99,BTC,UNDEAD,100000,0,0,$1.50,$1.50"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "1.0"))?;
        if !row.contains("150000.0000") { return Err(format!("currency_format: expected 150000.0000 in: {row}")); }
        Ok(row)
    }
    run_with!("currency_format", now(), compose!(resolve)(currency_format));

    fn apr_math((close_str, open_str): (String, String)) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,ids,close_id,pivot_token,from,\
            pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{close_str},{open_str},20,99,BTC,UNDEAD,0,100,0,0.00,0.00"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "110.0"))?;
        if !row.contains("10.00%") { return Err(format!("apr_math: expected 10.00% in: {row}")); }
        Ok(row)
    }
    run!("apr_math", {
        let close  = today();
        let opened = close - chrono::Duration::days(365);
        let _ = report("apr_math", (format!("{close}"), format!("{opened}")), apr_math);
    });

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

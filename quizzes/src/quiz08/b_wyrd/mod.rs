use clap::Parser;

use libs::{
   fetchers::calls::fetch_calls_table,
   tables::IxTable,
   types::util::Id
};
use book::{
    parse_args_add_banner,
    cli_utils::add_banner,
    currency::usd::{ USD, mk_usd },
    date_utils::parse_date,
    err_utils::ErrStr,
    num::floats::comma_floats::CommaFloat,
    num_utils::parse_num,
    parse_utils::parse_usd,
    string_utils::UppercaseString,
    table_utils::val,
    utils::get_env
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
pub fn parse_row(table: &IxTable, ix: usize, tx_id: &str, new_to_actual: f32)
       -> ErrStr<String> {
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
    let actual       = new_to_actual;
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
    use book::{
       parse_utils::{ parse_id, parse_str },
       string_utils::s,
       table_utils::ingest
    };

    pub fn make_table(raw: &str) -> ErrStr<IxTable> {
        let lines: Vec<String> = raw.lines().map(s).collect();
        ingest(parse_id, parse_str, parse_str, &lines, ",")
    }

    pub fn hdr() -> String {
       let a = "ix,close_date,opened,ids,close_id,pivot_token,from";
       let b = "pivot_amount,amount1,virtual,pivot_close_price";
       format!("{a},{b},proposed_close_price")
    }
}

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;
    use super::test_functions::{ make_table as tbl, hdr };
    use book::{
       date_utils::{ today, yesterday },
       err_utils::err_or,
       table_utils::row
    };

    fn mock_dusk_output() -> ErrStr<IxTable> {
        let dt = today();
        let y = yesterday();
        tbl(&format!("ix,opened,close_date,close_id,pool,ids,from,amount1,virtual,pivot_token,pivot_amount,pivot_close_price,proposed_close_price
1,{y},{dt},1,BTC+UNDEAD,20,UNDEAD,999999,0,BTC,0.01,$63997,$0.0002398
2,{y},{dt},2,ETH+UNDEAD,15,UNDEAD,999999,0,ETH,0.2,$1727.43,$0.0002398
3,{y},{dt},3,SOL+UNDEAD,10,UNDEAD,999999,0,SOL,9.5,$59.29,$0.0002398"))
    }

    fn mock_trade_row() -> ErrStr<IxTable> {
        let dt = format!("{}", today());
        let row = format!("1,{dt},{dt},20,99,BTC,UNDEAD,500,100,50,2.00,2.00");
        tbl(&format!("{}\n{row}", hdr()))
    }

    fn mock_losing_row() -> ErrStr<IxTable> {
        let dt = format!("{}", today());
        let row = format!("1,{dt},{dt},20,99,BTC,UNDEAD,0,100,50,0.00,0.00");
        tbl(&format!("{}\n{row}", hdr()))
    }

    fn mock_missing_dates() -> ErrStr<IxTable> {
       tbl("ix,amount1,virtual\n1,100,50")
    }

    fn test_parse_row(table: ErrStr<IxTable>, tx_id: &str, amt: f32)
          -> ErrStr<String> {
       parse_row(&table?, 1, tx_id, amt)
    }

    #[test] fn test_ix_in_bounds() -> ErrStr<()> {
        let table = mock_dusk_output()?;
        assert!(row(&table, &1).is_some());
        assert!(row(&table, &3).is_some());
        Ok(())
    }

    #[test] fn test_ix_out_of_bounds() -> ErrStr<()> {
        let table = mock_dusk_output()?;
        assert!(row(&table, &99).is_none());
        Ok(())
    }

    #[test] fn test_parse_row_ok() -> ErrStr<()> {
       let _ = test_parse_row(mock_dusk_output(), "tx1", 160.0)?;
       Ok(())
    }

    #[test] fn test_parse_row_err_on_missing_dates() -> ErrStr<()> {
        let row = test_parse_row(mock_missing_dates(), "tx1", 160.0);
        assert!(row.is_err());
        Ok(())
    }

    #[test] fn test_parse_row_field_count_matches_header() -> ErrStr<()> {
        let row_str = test_parse_row(mock_trade_row(), "tx1", 160.0)?;
        assert_eq!(header().split(',').count(), row_str.split(',').count());
        Ok(())
    }

    #[test] fn test_parse_row_passthrough_tx_id() -> ErrStr<()> {
        let row = test_parse_row(mock_trade_row(), "my-tx-abc", 160.0)?;
        assert!(row.contains("my-tx-abc"));
        Ok(())
    }

    #[test] fn test_parse_row_vol_calculation() -> ErrStr<()> {
        let row = test_parse_row(mock_trade_row(), "tx1", 160.0)?;
        assert!(row.contains("$1000.00"), "expected $1000.00 in: {row}");
        Ok(())
    }

    #[test] fn test_parse_row_gain_10_percent() -> ErrStr<()> {
        let row = test_parse_row(mock_trade_row(), "tx1", 160.0)?;
        assert!(row.contains("165.0000"), "expected 165.0000 in: {row}");
        Ok(())
    }

    #[test] fn test_parse_row_negative_gain() -> ErrStr<()> {
        let row = test_parse_row(mock_losing_row(), "tx1", 100.0)?;
        let datum = row.split(',').nth(12).unwrap_or("0");
        let gain: f32 =
           err_or(datum.parse(), &format!("Cannot parse {datum} to a float"))?;
                 
        assert!(gain < 0.0, "expected negative gain, got {gain}");
        Ok(())
    }

    #[test] fn test_parse_row_no_nan_or_inf() -> ErrStr<()> {
        let row = test_parse_row(mock_trade_row(), "tx1", 160.0)?;
        assert!(!row.contains("NaN") && !row.contains("inf"),
            "NaN or inf in output: {row}");
        Ok(())
    }

    #[test] fn test_parse_row_comma_formatted_new_to_actual() -> ErrStr<()> {
        let amt = "883,619.4538";
        let val: CommaFloat =
           err_or(amt.parse(), &format!("Cannot parse comma-float {amt}"))?;
        let row = test_parse_row(mock_trade_row(), "tx1", val.into())?;
        assert_eq!(header().split(',').count(), row.split(',').count(),
            "comma in new_to_actual broke field count: {row}");
        assert!(!row.contains("883,619"),
            "raw comma-formatted value leaked into CSV output: {row}");
        assert!(row.contains("883619"),
            "float value not in CSV output: {row}");
        Ok(())
    }

    fn btc_undead_dates(opened: NaiveDate, closed: NaiveDate, amt: f32)
          -> ErrStr<IxTable> {
        let daters = format!("20,99,BTC,UNDEAD,500,100,50,2.00,{amt}");
        let row = format!("1,{opened},{closed},{daters},{amt}");
        tbl(&format!("{}\n{row}", hdr()))
    }
    fn btc_undead(amt: f32) -> ErrStr<IxTable> {
        btc_undead_dates(today(), today(), amt)
    }

    #[test]
    fn test_parse_row_currency_fields_have_dollar_prefix() -> ErrStr<()> {
        let row = test_parse_row(btc_undead(3.0), "tx1", 160.0)?;
        let fields: Vec<&str> = row.split(',').collect();
        fn test_dollar_sign(hdr: &str, field: &str) {
           assert!(field.starts_with('$'), "{hdr} missing $: '{field}'");
        }
        test_dollar_sign("from_quote", &fields[5]);
        test_dollar_sign("vol", &fields[9]);
        test_dollar_sign("gain_total_usd", &fields[13]);
        Ok(())
    }

    #[test] fn test_truth_values_populated() -> ErrStr<()> {
        let row = test_parse_row(btc_undead(2.0), "tx1", 160.0)?;
        let fields: Vec<&str> = row.split(',').collect();
        assert_eq!(fields.len(), 16);
        for (i, f) in fields.iter().enumerate() {
            assert!(!f.is_empty(),
                    "field at index {i} is empty: full row: {row}");
        }
        Ok(())
    }
    
    #[test]
    fn test_parse_row_inverted_dates_returns_error() -> ErrStr<()> {
        let close  = today();
        let opened = close + chrono::Duration::days(365);
        let table = btc_undead_dates(close, opened, 2.0);
        let result = test_parse_row(table, "tx1", 160.0);
        assert!(result.is_err(), "expected Err for inverted dates, got Ok");
        let msg = result.unwrap_err();
        assert!(msg.contains("cannot compute APR"),
            "expected APR error message, got: {msg}");
        Ok(())
    }

    #[test] fn test_parse_row_err_on_empty_to_quote() -> ErrStr<()> {
        let dt = format!("{}", today());
        let table = tbl(&format!("{}\n1,{dt},{dt},500,100,50,2.00,", hdr()));
        let result = test_parse_row(table, "tx1", 160.0);
        assert!(result.is_err(),
                "expected Err when proposed_close_price is empty, got Ok");
        Ok(())
    }

    #[test] fn test_pool_path_format() -> ErrStr<()> {
        let table = mock_trade_row()?;
        let path = pool_path("close_pivots", &table, 1).unwrap();
        assert_eq!(path, "close_pivots/btc-undead.tsv");
        Ok(())
    }

    fn pivot_pool(prim: &str, piv: &str) -> ErrStr<IxTable> {
       let dt = today();
       let hdr_row =
          format!("{}\n1,{dt},{dt},20,99,{prim},{piv},0,0,0,0.00,0.00", hdr());
       tbl(&hdr_row)
    }

    #[test] fn test_pool_path_eth_undead() -> ErrStr<()> {
        let table = pivot_pool("ETH", "UNDEAD")?;
        assert_eq!(pool_path("cpp", &table, 1).unwrap(), "cpp/eth-undead.tsv");
        Ok(())
    }

    #[test] fn test_pool_path_btc_eth_alphabetical_order() -> ErrStr<()> {
        let table = pivot_pool("ETH", "BTC")?;
        assert_eq!(pool_path("geophf_is_grate", &table, 1).unwrap(),
            "geophf_is_grate/btc-eth.tsv");
        Ok(())
    }

    #[test] fn test_pool_path_undead_usdc_alphabetical_order() -> ErrStr<()> {
        let table = pivot_pool("USDC", "UNDEAD")?;
        assert_eq!(pool_path("dpcr", &table, 1).unwrap(),
            "dpcr/undead-usdc.tsv");
        Ok(())
    }

    #[test] fn missing_table_close_date_header() -> ErrStr<()> {
        let table = tbl("ix,pool,ids\n1,BTC,20")?;
        let row = parse_row(&table, 1, "tx_id", 1.0);
        assert!(row.is_err());
        Ok(())  
    }
}

//----- fn runoff_with_args -----------------------------------

/// Generates close pivot row from transaction id and calls-table
#[derive(Debug, Parser)]
#[command(name = "wyrd")]
#[command(version = "1.03")]
struct Args {
   /// Protocol to construct the close pivot, e.g.: PIVOT
   protocol: UppercaseString,

   /// path to close-pivot tables, e.g. data/pivots/close/raw
   path: String,

   /// close-pivot ix from the calls.tsv table, e.g.: 5
   ix: Id,

   /// transaction id of the close-pivot swap
   tx_id: String,

   /// The actual amount received in the close-pivot swap, e.g.: 1250.75
   amount: CommaFloat
}

pub async fn runoff_with_args() -> ErrStr<()> {
    let args = parse_args_add_banner!(Args);
    runoff_continuation(&args.protocol, &args.path, args.ix,
                        &args.tx_id, args.amount.into()).await
}

async fn runoff_continuation(protocol: &str, path: &str, ix: Id, tx_id: &str,
                             amount: f32) -> ErrStr<()> {
    let root_url = get_env(&format!("{protocol}_URL"))?;
    let calls = fetch_calls_table(&root_url).await?;
    println!("{}", header());
    println!("{}", parse_row(&calls, ix, tx_id, amount)?);
    println!("{}", pool_path(path, &calls, ix)?);
    Ok(())
}

// =====================================================
//----- FUNCTIONAL TESTS -------------------------------
// =====================================================

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
    use super::*;
    use super::test_functions::{ make_table, hdr };
    use paste::paste;
    use book::{
       create_testing,
       date_utils::today,
       utils::{ composer, resolve }
    };

    fn now() -> String { format!("{}", today()) }

    create_testing!("quiz08::b_wyrd");

    run!("parse_row", {
        let raw_data = 
        "ix,pool,open_pivots,last_pivot_on_dt,opened,ids,close_id,close_date,from,from_blockchain,amount1,virtual,quote1,val1,gain_10_percent,pivot_token,pivot_blockchain,pivot_close_price,pivot_amount,proposed_token,proposed_blockchain,proposed_close_price,proposed_amount,roi,apr
1,BTC+ETH,27,2026-05-07,2025-11-05,46,14,2026-05-19,ETH,Avalanche,0,0.1498,$3340.95,$500.47,0.16478,BTC,Avalanche,$76815.00,0.00467,ETH,Avalanche,$2116.94,0.169455,13.12%,24.56%";
        let table = make_table(raw_data)?;
        let row = parse_row(&table, 1, "asdf", 0.17)?;
        println!("result: {row} ");
    });

    fn test_row_dater(dater: &str, actual: f32) -> ErrStr<String> {
        let row = make_table(&format!("{}\n{dater}", hdr()))
                     .and_then(|t| parse_row(&t, 1, "tx_id", actual))?;
        if row.contains("NaN") || row.contains("inf") {
           Err(format!("apr_safety: NaN or inf in: {row}"))
        } else {
           Ok(row)
        }
    }
    fn test_row(amt: f32, actual: f32) -> impl Fn(String) -> ErrStr<String> {
       move |dt: String| {
          let dater =
             format!("1,{dt},{dt},20,99,BTC,UNDEAD,{amt},0,1,$1.50,$1.50");
          test_row_dater(&dater, actual)
       }
    }
    run_with!("apr_safety", now(), composer(resolve, test_row(0.0, 1.0)));
    run_with!("whale", now(), composer(resolve, test_row(150000000.0, 1.0)));
    run_with!("roi_zero_div", now(), composer(resolve, test_row(0.0, 0.0)));
    run_with!("column_count", now(), composer(resolve, test_row(0.0,120.0)));
    run_with!("currency_format", now(),
              composer(resolve, test_row(100000.0, 1.0)));
    run!("undead_zero_precision", {
       let dt = now();
       let dater = 
          format!("1,{dt},{dt},20,99,BTC,UNDEAD,1000,500,100,0.0025,0.0025");
       let ans = test_row_dater(&dater, 650.0)?;
       println!("Close pivot with UNDEAD prices:\n{ans}");
    });
}


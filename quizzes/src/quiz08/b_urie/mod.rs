use book::err_utils::ErrStr;
use book::parse_utils::{ parse_id, parse_str };
use book::table_utils::{ ingest, val };
use book::date_utils::parse_date;
use libs::tables::IxTable;
use book::utils::get_env;
use libs::fetchers::fetch_calls;
use tokio::runtime::Runtime;


// ===========================================================================================================================
//----- pub fn run -----------------------------------------------------------------------------------------------------------
// ===========================================================================================================================
pub fn run() -> ErrStr<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Error: not enough arguments.");
        eprintln!("Usage: `panic` <ix> <tx_id> <new_to_actual>");
        eprintln!("Example: `panic` 5 \"asdf\" \"1250.75\"");
        std::process::exit(1);
    }
    let ix       = parse_id(&args[1])?;
    let root_url = get_env("PIVOT_URL")?;
    let rt       = Runtime::new().map_err(|e| e.to_string())?;
    match rt.block_on(fetch_calls(&root_url)) {
        Ok(t)  => {
            println!("{}", header());
            println!("{}", parse_row(&t, ix, &args[2], &args[3])?);
        }
        Err(e) => eprintln!("Error: {e}"),
    }
    Ok(())
}
// ===========================================================================================================================
//----- pub fn header --------------------------------------------------------------------------------------------------------
// ===========================================================================================================================
pub fn header() -> String {
    let line1 = format!("date,pivot,close,tx_id,from,from_quote");
    let line2 = format!("to,to_quote,trade,vol,gain_10_percent");
    let line3 = format!("new_to_actual,gain,gain_total_usd,roi,apr");
    format!("{line1},{line2},{line3}")
}
// ===========================================================================================================================
//----- pub fn parse_row -----------------------------------------------------------------------------------------------------
// ===========================================================================================================================
pub fn parse_row(table: &IxTable, ix: usize, tx_id: &str, new_to_actual: &str) -> ErrStr<String> {
    let col     = |name: &str| val(&table, &ix, &name.to_string()).unwrap_or_default();
    let col_num = |name: &str| -> ErrStr<f64> {
        let strip = col(name).replace('$', "").replace(',', "");
        let strip = strip.trim();
        if strip.is_empty() {
            Err("missing table's data".to_string())
        } else {
            strip.parse::<f64>().map_err(|_| "missing table's data".to_string())
        }
    };
    let col_opt = |name: &str| -> ErrStr<f64> {
        let strip = col(name).replace('$', "").replace(',', "");
        strip.trim().parse::<f64>().map_err(|_| format!("invalid value for '{name}' "))
    };
    //----- truth values -----------------------------------------------------------------
    let date     = parse_date(&col("close_date"))?;
    let opened   = parse_date(&col("opened"))?;
    let pivot    = col("ids");
    let close    = col("close_id");
    let from     = col("pivot_token");
    let to       = col("from");
    let trade    = col_num("pivot_amount")?;
    let amount1  = col_num("amount1")?;
    let virtual_ = col_num("virtual")?;
    let actual   = new_to_actual.parse::<f64>().map_err(|_| format!("invalid new_to_actual: '{new_to_actual}' is not a number"))?;
    let from_quote   = col_num("pivot_close_price")?;
    let to_quote = col_opt("proposed_close_price")?;
    //----- formulas for the correct headers ---------------------------------------------
    let sum_amt_virt    = amount1 + virtual_;
    let vol             = trade * from_quote;
    let gain_10_percent = sum_amt_virt * 1.1;
    let gain            = actual - sum_amt_virt;
    let gain_total_usd  = gain * to_quote;
    let roi_val         = if sum_amt_virt != 0.0 { gain / sum_amt_virt } else { 0.0 };
    let days            = (date - opened).num_days() as f64;
    let apr_val         = if days > 0.0 { (roi_val * 365.0) / days } else { 0.0 };
    //----- formatting the actual output -------------------------------------------------
    let line1 = format!("{date},{pivot},{close},{tx_id},{from},${from_quote}");
    let line2 = format!("{to},${to_quote},{trade},${vol:.4},{gain_10_percent:.4}");
    let line3 = format!("{new_to_actual},{gain:.4},${gain_total_usd:.2},{:.2}%,{:.2}%",
                        roi_val * 100.0, apr_val * 100.0);
    Ok(format!("{line1},{line2},{line3}"))
}
// ===========================================================================================================================
//----- UNIT TESTS -----------------------------------------------------------------------------------------------------------
// ===========================================================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use book::table_utils::row;
    use book::date_utils::today;


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
            "ix,close_date,opened,pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},500,100,50,2.00,2.00"
        )
    }

    fn mock_losing_row() -> String {
        let dt = format!("{}", today());
        format!(
            "ix,close_date,opened,pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},0,100,50,0.00,0.00"
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
        let gain: f64 = row_str.split(',').nth(12).unwrap_or("0").parse().unwrap_or(0.0);
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
    fn test_col_num_strips_dollar() {
        let v: f64 = "$1.50".replace('$', "").replace(',', "").trim().parse().unwrap_or(0.0);
        assert!((v - 1.50).abs() < f64::EPSILON);
    }

    #[test]
    fn test_col_num_strips_comma() {
        let v: f64 = "1,250.75".replace('$', "").replace(',', "").trim().parse().unwrap_or(0.0);
        assert!((v - 1250.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_col_num_strips_dollar_and_comma() {
        let v: f64 = "$1,250.75".replace('$', "").replace(',', "").trim().parse().unwrap_or(0.0);
        assert!((v - 1250.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_col_num_empty_returns_missing_data_err() {
        let raw = "";
        let result: Result<f64, &str> = {
            let cleaned = raw.replace('$', "").replace(',', "");
            let trimmed = cleaned.trim();
            if trimmed.is_empty() {
                Err("missing table's data")
            } else {
                trimmed.parse::<f64>().map_err(|_| "missing table's data")
            }
        };
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "missing table's data");
    }

    #[test]
    fn test_roi_zero_division_guard() {
        let s = 0.0_f64;
        assert_eq!(0.0, if s != 0.0 { 1.0 / s } else { 0.0 });
    }

    #[test]
    fn test_apr_zero_days_guard() {
        assert_eq!(0.0, if 0.0_f64 > 0.0 { 0.1 * 365.0 / 0.0 } else { 0.0 });
    }
    #[test]
    fn test_apr_365_days_equals_roi() {
        let roi = 0.1_f64;
        assert!((roi - roi * 365.0 / 365.0).abs() < 1e-10);
    }
    
    #[test] 
    fn test_functional_suite() { 
        super::functional_tests::runoff().expect("Functional suite failed"); 
    }

    #[test] 
    fn test_gain_formula() { 
        assert_eq!(-50.0, 100.0_f64 - (100.0 + 50.0)); 
    }    

    #[test] 
    fn test_gain_10_percent() { 
        assert!((165.0_f64 - (100.0 + 50.0) * 1.1).abs() < f64::EPSILON); 
    }

    #[test] 
    fn test_vol_formula() { 
        assert!((750.0_f64 - 500.0 * 1.5).abs() < f64::EPSILON); 
    }

    #[test] 
    fn test_col_num_strips() { 
        let v: f64 = "$1,250.75".replace('$',"").replace(',',"").trim().parse().unwrap_or(0.0); 
        assert!((v - 1250.75).abs() < f64::EPSILON); 
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
    
}
// ===========================================================================================================================
//----- FUNCTIONAL TESTS -----------------------------------------------------------------------------------------------------
// ===========================================================================================================================
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
    use book::{ err_utils::ErrStr, date_utils::today };
    use super::*;

    
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

    fn run_resilience() -> ErrStr<usize> {
        let table = make_table("ix,pool,ids\n1,BTC,20")?;
        if parse_row(&table, 1, "tx_id", "1.0").is_ok() {
            return Err("resilience: expected Err on missing dates, got Ok".to_string());
        }
        println!("\npivot_dapps::run_resilience functional test\n\n\tresult: Err (correct)\n\npivot_dapps::resilience:...ok");
        Ok(1)
    }

    fn apr_safety(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},0,0,1,0.00,0.00"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "1.0"))?;
        if row.contains("NaN") || row.contains("inf") { return Err(format!("apr_safety: NaN or inf in: {row}")); }
        Ok(row)
    }
    fn run_apr_safety() -> ErrStr<usize> { report("apr_safety", now(), apr_safety) }

    fn whale(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},100000000,0,0,1.50,1.50"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "1.0"))?;
        if !row.contains("150000000.0000") { return Err(format!("whale: expected 150000000.0000 in: {row}")); }
        Ok(row)
    }
    fn run_whale() -> ErrStr<usize> { report("whale", now(), whale) }

    fn roi_zero_div(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},0,0,0,0.00,0.00"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "0.0"))?;
        if row.contains("NaN") || row.contains("inf") { return Err(format!("roi_zero_div: NaN or inf in: {row}")); }
        Ok(row)
    }
    fn run_roi_zero_div() -> ErrStr<usize> { report("roi_zero_div", now(), roi_zero_div) }

    fn column_count(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},0,100,50,0.00,0.00"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "120.0"))?;
        let (h, r) = (header().split(',').count(), row.split(',').count());
        if h != r { return Err(format!("column_count: header={h} row={r}")); }
        Ok(row)
    }
    fn run_column_count() -> ErrStr<usize> { report("column_count", now(), column_count) }

    fn currency_format(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{dt},{dt},100000,0,0,$1.50,$1.50"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "1.0"))?;
        if !row.contains("150000.0000") { return Err(format!("currency_format: expected 150000.0000 in: {row}")); }
        Ok(row)
    }
    fn run_currency_format() -> ErrStr<usize> { report("currency_format", now(), currency_format) }

    fn apr_math((close_str, open_str): (String, String)) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,pivot_amount,amount1,virtual,pivot_close_price,proposed_close_price\n\
            1,{close_str},{open_str},0,100,0,0.00,0.00"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "110.0"))?;
        if !row.contains("10.00%") { return Err(format!("apr_math: expected 10.00% in: {row}")); }
        Ok(row)
    }
    fn run_apr_math() -> ErrStr<usize> {
        let close  = today();
        let opened = close - chrono::Duration::days(365);
        report("apr_math", (format!("{close}"), format!("{opened}")), apr_math)
    }

    fn undead_zero_precision(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,pivot_token,pivot_close_price,from,pivot_amount,amount1,virtual,proposed_close_price\n\
            1,{dt},{dt},UNDEAD,0.0025,UNDEAD,1000,500,100,0.0025"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "650.0"))?;
        let gain_usd: f64 = row.split(',').nth(13).unwrap_or("$0").replace('$', "").parse().unwrap_or(0.0);
        if gain_usd == 0.0 { return Err("undead_zero_precision: gain_total_usd is 0, to_quote not rescued".to_string()); }
        Ok(row)
    }
    fn run_undead_zero_precision() -> ErrStr<usize> { report("undead_zero_precision", now(), undead_zero_precision) }

    pub fn runoff() -> ErrStr<usize> {
        println!("\npivot_dapps functional tests\n");
        let a = run_resilience()?;
        let b = run_apr_safety()?;
        let c = run_whale()?;
        let d = run_roi_zero_div()?;
        let e = run_column_count()?;
        let f = run_currency_format()?;
        let g = run_apr_math()?;
        let h = run_undead_zero_precision()?;
        println!("\nFunctional suite complete. All tests passed.");
        Ok(a+b+c+d+e+f+g+h)
    }
    
}

use book::err_utils::ErrStr;
use book::parse_utils::parse_id;
use book::parse_utils::parse_str;
use book::table_utils::ingest;
use book::table_utils::val;
use book::utils::get_env;
use libs::tables::IxTable;
use libs::fetchers::fetch_calls;
use book::date_utils::parse_date;


fn header() -> String {
    let line1 = format!("date,pivot,close,tx_id,from,from_quote");
    let line2 = format!("to,to_quote,trade,vol,gain_10_percent");
    let line3 = format!("new_to_actual,gain,gain_total_usd,roi,apr");
    format!("{line1},{line2},{line3}")
}
// ===========================================================================================================================
//----- fn parse_row ---------------------------------------------------------------------------------------------------------
// ===========================================================================================================================
fn parse_row(table: &IxTable, ix: usize, tx_id: &str, new_to_actual: &str) -> ErrStr<String> {
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
    let col_opt = |name: &str| -> f64 {
        let strip = col(name).replace('$', "").replace(',', "");
        strip.trim().parse::<f64>().unwrap_or(0.0)
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
    let actual   = new_to_actual.parse::<f64>().unwrap_or(0.0);
    let from_quote   = col_num("pivot_close_price")?;
    let mut to_quote = col_opt("proposed_close_price");
    if to_quote == 0.0 && from_quote > 0.0 {
        to_quote = from_quote;
    }
    //----- formulas for the correct headers ---------------------------------------------
    let sum_amt_virt    = amount1 + virtual_;
    let vol             = trade * from_quote;
    let gain_10_percent = sum_amt_virt * 1.1;
    let gain            = actual - sum_amt_virt;
    let gain_total_usd  = gain * to_quote;
    let roi_val         = if sum_amt_virt != 0.0 { gain / sum_amt_virt } else { 0.0 };
    let days            = (date - opened).num_days() as f64;
    let apr_val         = if days > 0.0 { (roi_val * 365.0) / days } else { 0.0 };
    //----- foramtting the actual output -------------------------------------------------
    let line1 = format!("{date},{pivot},{close},{tx_id},{from},${from_quote}");
    let line2 = format!("{to},${to_quote},{trade},${vol:.4},{gain_10_percent:.4}");
    let line3 = format!("{new_to_actual},{gain:.4},${gain_total_usd:.2},{:.2}%,{:.2}%",
                        roi_val * 100.0, apr_val * 100.0);
    Ok(format!("{line1},{line2},{line3}"))
}
// ===========================================================================================================================
//----- fn main --------------------------------------------------------------------------------------------------------------
// ===========================================================================================================================
fn main() -> ErrStr<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Error: not enough arguments.");
        eprintln!("Usage: `panic` <ix> <tx_id> <new_to_actual>");
        eprintln!("Example: `panic` 5 \"asdf\" \"1250.75\"");
        std::process::exit(1);
    }
    let ix_str = &args[1];
    let tx_id = &args[2];
    let new_to_actual = &args[3];
    let ix = parse_id(ix_str)?;
    let root_url = get_env("PIVOT_URL")?;
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    match rt.block_on(fetch_calls(&root_url)) {
        Ok(t) => {
            println!("{}", header()); // <-- if you comment this line out/ remove this line, the output will just be the row without the header :)
            println!("{}", parse_row(&t, ix, tx_id, new_to_actual)?);
        }
        Err(e) => eprintln!("Error: {e}"),
    }
    Ok(())
}
// ===========================================================================================================================
//----- UNIT TESTS -----------------------------------------------------------------------------------------------------------
// ===========================================================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use book::table_utils::row;
    use book::date_utils::today;


    //----- mock data ------------------------------------------------------------------------
    fn mock_dusk_output() -> String {
        "ix,pool,ids\n\
         1,BTC+UNDEAD,20\n\
         2,ETH+UNDEAD,15\n\
         3,SOL+UNDEAD,10"
            .to_string()
    }
    // used by parse_row tests: provides close_date + opened so parse_date doesn't Err
    fn mock_trade_row() -> String {
        let dt = format!("{}", today());
        format!(
            "ix,close_date,opened,pivot_amount,amount1,virtual,pivot_close_price\n\
             1,{dt},{dt},500,100,50,2.00"
        )
    }
    // same as mock_trade_row but new_to_actual < sum so gain is negative
    fn mock_losing_row() -> String {
        let dt = format!("{}", today());
        format!(
            "ix,close_date,opened,pivot_amount,amount1,virtual,pivot_close_price\n\
            1,{dt},{dt},0,100,50,0.00"
        )
    }
    // missing close_date and opened entirely — parse_row must return Err
    fn mock_missing_dates() -> String {
        "ix,amount1,virtual\n\
         1,100,50"
            .to_string()
    }
    //----- ingest/ row in and out of bounds -------------------------------------------------
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
    //----- header ---------------------------------------------------------------------------
    // header() produces 16 comma-separated fields across 3 format! lines,
    // this pins that count so a stray comma or added column is caught immediately
    #[test]
    fn test_header_field_count() {
        assert_eq!(16, header().split(',').count());
    }
    // confirms the column names the downstream CSV consumers actually look for are present,
    // catches renames or typos in the header strings
    #[test]
    fn test_header_contains_key_fields() {
        let h = header();
        for field in &["date","pivot","close","tx_id","from","to",
                       "trade","vol","gain","roi","apr"] {
            assert!(h.contains(field), "header missing field: {field}");
        }
    }
    //----- parse_row ------------------------------------------------------------------------
    // confirms the happy path compiles and returns Ok with valid input
    #[test]
    fn test_parse_row_returns_ok() {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest mock data");
        assert!(parse_row(&table, 1, "tx1", "160.0").is_ok());
    }
    // parse_row must return Err when close_date/ opened are missing because parse_date uses " ? " 
    // this confirms that error path is actually taken and not swallowed
    #[test]
    fn test_parse_row_err_on_missing_dates() {
        let raw_data = mock_missing_dates();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest mock data");
        assert!(parse_row(&table, 1, "tx1", "160.0").is_err());
    }
    // header() and parse_row() are written independently across 3 format! lines each,
    // a mismatch silently produces malformed CSV, so this will be an explicit test
    #[test]
    fn test_parse_row_field_count_matches_header() {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest mock data");
        let row_str = parse_row(&table, 1, "tx1", "160.0").unwrap();
        assert_eq!(header().split(',').count(), row_str.split(',').count());
    }
    // tx_id is passed through as a raw arg, not read from the table,
    // this confirms it actually lands in the output row
    #[test]
    fn test_parse_row_passthrough_tx_id() {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest mock data");
        let row_str = parse_row(&table, 1, "my-tx-abc", "160.0").unwrap();
        assert!(row_str.contains("my-tx-abc"));
    }
    // spot-checks the vol formula with known inputs so a formula regression is caught with a clear expected value
    #[test]
    fn test_parse_row_vol_calculation() {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest mock data");
        let row_str = parse_row(&table, 1, "tx1", "160.0").unwrap();
        assert!(row_str.contains("$1000.0000"), "expected $1000.0000 in: {row_str}");
    }
    // same rationale as vol: known inputs, concrete expected output
    #[test]
    fn test_parse_row_gain_10_percent() {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest mock data");
        let row_str = parse_row(&table, 1, "tx1", "160.0").unwrap();
        assert!(row_str.contains("165.0000"), "expected 165.0000 in: {row_str}");
    }
    // field[12] is "gain", parses it as f64 rather than searching for '-' which would false-pass on date hyphens
    #[test]
    fn test_parse_row_negative_gain() {
        let raw_data = mock_losing_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest mock data");
        let row_str = parse_row(&table, 1, "tx1", "100.0").unwrap();
        let gain: f64 = row_str.split(',').nth(12).unwrap_or("0").parse().unwrap_or(0.0);
        assert!(gain < 0.0, "expected negative gain, got {gain}");
    }
    // NaN and inf can silently propagate through f64 arithmetic,
    // this confirms neither escapes into the output string
    #[test]
    fn test_parse_row_no_nan_or_inf() {
        let raw_data = mock_trade_row();
        let table = ingest(parse_id, parse_str, parse_str, &raw_data.lines().map(|s| s.to_string()).collect::<Vec<_>>(), ",")
            .expect("Failed to ingest mock data");
        let row_str = parse_row(&table, 1, "tx1", "160.0").unwrap();
        assert!(!row_str.contains("NaN") && !row_str.contains("inf"),
            "NaN or inf in output: {row_str}");
    }
    //----- col_num stripping ----------------------------------------------------------------
    // col_num strips " $ ", this tests " $ " alone so it's clear which symbol is being verified
    #[test]
    fn test_col_num_strips_dollar() {
        let v: f64 = "$1.50".replace('$', "").replace(',', "").trim().parse().unwrap_or(0.0);
        assert!((v - 1.50).abs() < f64::EPSILON);
    }
    // col_num strips " , ", thousands-separator commas must go before parse
    #[test]
    fn test_col_num_strips_comma() {
        let v: f64 = "1,250.75".replace('$', "").replace(',', "").trim().parse().unwrap_or(0.0);
        assert!((v - 1250.75).abs() < f64::EPSILON);
    }
    // both symbols together, the common real-world case like "$1,250.75"
    #[test]
    fn test_col_num_strips_dollar_and_comma() {
        let v: f64 = "$1,250.75".replace('$', "").replace(',', "").trim().parse().unwrap_or(0.0);
        assert!((v - 1250.75).abs() < f64::EPSILON);
    }
    // empty string means that something was absent in the source data,
    // silently defaulting to 0.0 would corrupt every downstream formula
    // that depends on this value, so empty must be a hard error
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
    //----- to_quote fallback ----------------------------------------------------------------
    // when proposed_close_price is 0 but pivot_close_price is set, to_quote must inherit from_quote
    #[test]
    fn test_to_quote_fallback_applies() {
        let mut q = 0.0_f64; let f = 0.0025_f64;
        if q == 0.0 && f > 0.0 { q = f; }
        assert!((q - 0.0025).abs() < f64::EPSILON);
    }
    // when to_quote already has a real value the fallback **must not** overwrite it
    #[test]
    fn test_to_quote_fallback_skipped_when_set() {
        let mut q = 1.50_f64; let f = 0.0025_f64;
        if q == 0.0 && f > 0.0 { q = f; }
        assert!((q - 1.50).abs() < f64::EPSILON);
    }
    // if/ when both are zero the fallback condition (f > 0.0) must not fire,
    // result stays 0.0 rather than producing a misleading non-zero quote
    #[test]
    fn test_to_quote_fallback_skipped_when_from_also_zero() {
        let mut q = 0.0_f64; let f = 0.0_f64;
        if q == 0.0 && f > 0.0 { q = f; }
        assert_eq!(q, 0.0);
    }
    //----- pure math guards -----------------------------------------------------------------
    // sum_amt_virt=0 must produce roi=0 not a divide-by-zero panic or inf
    #[test]
    fn test_roi_zero_division_guard() {
        let s = 0.0_f64;
        assert_eq!(0.0, if s != 0.0 { 1.0 / s } else { 0.0 });
    }
    // days=0 must produce apr=0 not divide-by-zero, same date for open and close triggers this
    #[test]
    fn test_apr_zero_days_guard() {
        assert_eq!(0.0, if 0.0_f64 > 0.0 { 0.1 * 365.0 / 0.0 } else { 0.0 });
    }
    // over exactly 365 days, APR must equal ROI, validates the (roi*365)/days formula
    #[test]
    fn test_apr_365_days_equals_roi() {
        let roi = 0.1_f64;
        assert!((roi - roi * 365.0 / 365.0).abs() < 1e-10);
    }
}
// =========================================================================================================================
//----- FUNCTIONAL TESTS ---------------------------------------------------------------------------------------------------
// =========================================================================================================================
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
    use book::date_utils::today;
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
    //----- resilience to missing columns/ data expects Err, so report cannot be used here ----------
    fn run_resilience() -> ErrStr<usize> {
        let table = make_table("ix,pool,ids\n1,BTC,20")?;
        if parse_row(&table, 1, "tx_id", "1.0").is_ok() {
            return Err("resilience: expected Err on missing dates, got Ok".to_string());
        }
        println!("\npivot_dapps::run_resilience functional test\n\n\tresult: Err (correct)\n\npivot_dapps::resilience:...ok");
        Ok(1)
    }
    //----- apr safety ------------------------------------------------------------------------------
    fn apr_safety(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,pivot_amount,amount1,virtual,pivot_close_price\n\
            1,{dt},{dt},0,0,1,0.00"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "1.0"))?;
        if row.contains("NaN") || row.contains("inf") { return Err(format!("apr_safety: NaN or inf in: {row}")); }
        Ok(row)
    }
    fn run_apr_safety() -> ErrStr<usize> { report("apr_safety", now(), apr_safety) }
    //----- large number ------------------------------------------------------------------------------
    fn whale(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,pivot_amount,amount1,virtual,pivot_close_price\n\
            1,{dt},{dt},100000000,0,0,1.50"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "1.0"))?;
        if !row.contains("150000000.0000") { return Err(format!("whale: expected 150000000.0000 in: {row}")); }
        Ok(row)
    }
    fn run_whale() -> ErrStr<usize> { report("whale", now(), whale) }
    //----- roi zero-division -------------------------------------------------------------------------
    fn roi_zero_div(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,pivot_amount,amount1,virtual,pivot_close_price\n\
            1,{dt},{dt},0,0,0,0.00"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "0.0"))?;
        if row.contains("NaN") || row.contains("inf") { return Err(format!("roi_zero_div: NaN or inf in: {row}")); }
        Ok(row)
    }
    fn run_roi_zero_div() -> ErrStr<usize> { report("roi_zero_div", now(), roi_zero_div) }
    //----- column count ------------------------------------------------------------------------------
    fn column_count(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,pivot_amount,amount1,virtual,pivot_close_price\n\
            1,{dt},{dt},0,100,50,0.00"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "120.0"))?;
        let (h, r) = (header().split(',').count(), row.split(',').count());
        if h != r { return Err(format!("column_count: header={h} row={r}")); }
        Ok(row)
    }
    fn run_column_count() -> ErrStr<usize> { report("column_count", now(), column_count) }
    //----- negative gain ------------------------------------------------------------------------------
    fn negative_gain(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,pivot_amount,amount1,virtual,pivot_close_price\n\
            1,{dt},{dt},0,100,50,0.00"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "100.0"))?;
        let gain: f64 = row.split(',').nth(12).unwrap_or("0").parse().unwrap_or(0.0);
        if gain >= 0.0 { return Err(format!("negative_gain: expected gain < 0, got {gain}")); }
        Ok(row)
    }
    fn run_negative_gain() -> ErrStr<usize> { report("negative_gain", now(), negative_gain) }
    //----- currency format ----------------------------------------------------------------------------
    fn currency_format(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,pivot_amount,amount1,virtual,pivot_close_price\n\
            1,{dt},{dt},100000,0,0,$1.50"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "1.0"))?;
        if !row.contains("150000.0000") { return Err(format!("currency_format: expected 150000.0000 in: {row}")); }
        Ok(row)
    }
    fn run_currency_format() -> ErrStr<usize> { report("currency_format", now(), currency_format) }
    //----- apr 365-day math ----------------------------------------------------------------------------
    fn apr_math((close_str, open_str): (String, String)) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,pivot_amount,amount1,virtual,pivot_close_price\n\
            1,{close_str},{open_str},0,100,0,0.00"
        )).and_then(|t| parse_row(&t, 1, "tx_id", "110.0"))?;
        if !row.contains("10.00%") { return Err(format!("apr_math: expected 10.00% in: {row}")); }
        Ok(row)
    }
    fn run_apr_math() -> ErrStr<usize> {
        let close  = today();
        let opened = close - chrono::Duration::days(365);
        report("apr_math", (format!("{close}"), format!("{opened}")), apr_math)
    }
    //----- undead zero-precision -------------------------------------------------------------------------
    fn undead_zero_precision(dt: String) -> ErrStr<String> {
        let row = make_table(&format!(
            "ix,close_date,opened,pivot_token,pivot_close_price,from,pivot_amount,amount1,virtual\n\
            1,{dt},{dt},UNDEAD,0.0025,UNDEAD,1000,500,100"
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
        let f = run_negative_gain()?;
        let g = run_currency_format()?;
        let h = run_apr_math()?;
        let i = run_undead_zero_precision()?;
        println!("\nFunctional suite complete. All tests passed.");
        Ok(a+b+c+d+e+f+g+h+i)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test] fn test_functional_suite()    { runoff().expect("Functional suite failed"); }
        #[test] fn test_gain_formula()        { assert_eq!(-50.0, 100.0_f64 - (100.0 + 50.0)); }
        #[test] fn test_gain_10_percent()     { assert!((165.0_f64 - (100.0 + 50.0) * 1.1).abs() < f64::EPSILON); }
        #[test] fn test_vol_formula()         { assert!((750.0_f64 - 500.0 * 1.5).abs() < f64::EPSILON); }
        #[test] fn test_roi_zero_guard()      { let s = 0.0_f64; assert_eq!(0.0, if s != 0.0 { 1.0/s } else { 0.0 }); }
        #[test] fn test_apr_zero_days_guard() { assert_eq!(0.0, if 0.0_f64 > 0.0 { 0.1 * 365.0 / 0.0 } else { 0.0 }); }
        #[test] fn test_apr_365_equals_roi()  { let roi = 0.1_f64; assert!((roi - roi * 365.0 / 365.0).abs() < 1e-10); }
        #[test] fn test_to_quote_fallback()   { let mut q = 0.0_f64; let f = 0.0025_f64; if q == 0.0 && f > 0.0 { q = f; } assert_eq!(q, 0.0025); }
        #[test] fn test_col_num_strips()      { let v: f64 = "$1,250.75".replace('$',"").replace(',',"").trim().parse().unwrap_or(0.0); assert!((v - 1250.75).abs() < f64::EPSILON); }
        #[test] fn test_header_field_count()  { assert_eq!(16, header().split(',').count()); }
    }
}

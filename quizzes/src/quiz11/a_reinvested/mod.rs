use clap::Parser;
use csv::ReaderBuilder;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use book::{
        parse_args_add_banner,
        cli_utils::generate_banner,
        err_utils::ErrStr,
        parse_utils::parse_id,
        string_utils::plural,
        utils::get_env,
        num::floats::comma_floats::CommaFloat
};
use libs::{ 
        investor_rows::{
            chat_id_for, 
            is_ragged_row, 
            parse_bool_cell, 
            send_telegram, 
            SendFuture
        }
};

//============================================================================
//----- CSV Row Parsing ------------------------------------------------------
//============================================================================
#[derive(Debug)]
pub struct InvestorRow {
    pub name:    String,
    pub amount:  f32,
    pub primary: String,
    pub pivot:   String,
    pub pivots:  String,
    pub url:     String,
    pub send:    bool,
    pub flipped: bool,
}
#[serde_as]
#[derive(Debug, Deserialize)]
struct PivotRecord {
    name: String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "amount reinvested")]
    amount_reinvested: CommaFloat,
    primary: String,
    pivot: String,
    #[serde(rename = "number of pivots closed")]
    pivots: String,
    #[serde(rename = "tweet url")]
    tweet_url: String,
    #[serde(rename = "send?")]
    send: String,
    flipped: String,
}

/// Returns `Ok(None)` only for rows where amount reinvested == 0 (handled
/// by `distributed`, not `reinvested`). Returns `Err` for malformed data.
/// Structural issues (blank lines, ragged/short rows, the header row) never
/// reach this function — the TSV reader in `process_tsv` filters them out.
fn parse_row(record: &PivotRecord) -> ErrStr<Option<InvestorRow>> {
    let name    = record.name.trim();
    let amount: f32 = record.amount_reinvested.into();
    let primary = record.primary.trim();
    let pivot   = record.pivot.trim();
    let pivots  = record.pivots.trim();
    let url     = record.tweet_url.trim();

    if amount == 0.0 {
        return Ok(None);
    }

    let send    = parse_bool_cell("send", &record.send)?;
    let flipped = parse_bool_cell("flipped", &record.flipped)?;

    Ok(Some(InvestorRow {
        name:    name.to_string(),
        amount,
        primary: primary.to_string(),
        pivot:   pivot.to_string(),
        pivots:  pivots.to_string(),
        url:     url.to_string(),
        send,
        flipped,
    }))
}

//============================================================================
//----- Message Building -------------------------------------------------
//============================================================================
pub fn build_message(row: &InvestorRow) -> ErrStr<String> {
    let prim = &row.primary;
    let piv  = &row.pivot;
    let pool = format!("{prim}+{piv}");
    let trade = format!("{prim}-on-{piv}");
    let reinvested = if row.flipped { piv.as_str() } else { prim.as_str() };
    let n      = parse_id(&row.pivots)?;
    let noun   = format!("{trade} pivot");
    let pivots = if n == 1 { noun.clone() } else { plural(n, &noun) };
    Ok(format!(
        "I close {pivots} (see tweet: {url}). \
         I reinvest {amount} {reinvested} into the {pool} pivot pool for you.",
        url    = row.url,
        amount = row.amount,
    ))
}

//============================================================================
//----- Core: process all rows in one pass -----------------------------------
//============================================================================
pub async fn process_tsv<F>(tsv_path: &str, global_send: bool, send_fn: F)
   -> ErrStr<()> where F: for<'a> Fn(&'a str, i64, &'a str) -> SendFuture<'a> {
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .from_path(tsv_path)
        .map_err(|e| format!("cannot read '{tsv_path}': {e}"))?;

    for result in rdr.deserialize::<PivotRecord>() {
        let record = match result {
            Ok(r) => r,
            Err(e) if is_ragged_row(&e) => continue,
            Err(e) => return Err(format!("malformed row in '{tsv_path}': {e}")),
        };

        let Some(row) = parse_row(&record)? else { continue };

        let msg = build_message(&row)?;
        println!("[{}] {msg}", row.name);

        if global_send && row.send {
            let bot_token = get_env("REINVESTED_BOT")?;
            let chat_id   = chat_id_for(&row.name)?;
            send_fn(&bot_token, chat_id, &msg).await?;
        }
    }
    Ok(())
}

/// Sends reinvestment message to investors
///
/// The investors and their reinvestments are listed in CSV file
#[derive(Debug, Parser)]
#[command(name = "reinvested")]
#[command(version = "2.04")]
struct Args {
   /// The path to the list of the investors and their distributions
   tsv_path: String,

   /// Send a telegram? (yes/no)
   send: String
}

pub async fn runoff_with_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   let send = parse_bool_cell("send", &args.send)?;
   process_tsv(&args.tsv_path, send, |tok, id, txt| {
                Box::pin(send_telegram(tok, id, txt))
   }).await
}

//============================================================================
//----- UNIT TESTS -----------------------------------------------------------
//============================================================================
#[cfg(test)]
mod unit_tests {
    use super::*;
    use libs::investor_rows::test_functions::deserialize_test_row;


    // ---- helpers -----------------------------------------------------------
    fn make_row(
        name: &str, amount: &str, send: &str, flipped: &str,
    ) -> String {
        // cols: 0=name 1=reinvested% 2=precentage 3=amount_reinvested 4=amount_distributed
        //        5=primary 6=pivot 7=usd 8=pivots 9=tweet_url 10=tx_url 11=send 12=flipped
        let ans = format!(
            "{name}\t100%\t3.46%\t{amount}\t0\tBTC\tUNDEAD\t$12.04\t15\t\
             https://x.com/pivocateur/status/2069591552733712719\t\
             https://snowtrace.io/tx/0xabc\t{send}\t{flipped}"
        );
        ans
    }

    fn make_investor(name: &str, amount: f32, send: bool, flipped: bool) -> InvestorRow {
        InvestorRow {
            name:    name.to_string(),
            amount,
            primary: "BTC".to_string(),
            pivot:   "UNDEAD".to_string(),
            pivots:  "15".to_string(),
            url:     "https://x.com/pivocateur".to_string(),
            send,
            flipped,
        }
    }

    // Now delegates the raw TSV -> PivotRecord step to the shared helper in
    // libs::investor_rows::test_helpers; only the domain-specific parse_row
    // step stays local, since InvestorRow/PivotRecord are reinvested-only.
    fn parse_test_row(line: &str) -> ErrStr<Option<InvestorRow>> {
        match deserialize_test_row::<PivotRecord>(line)? {
            None         => Ok(None),
            Some(record) => parse_row(&record),
        }
    }

    // ---- parse_row ---------------------------------------------------------
    #[test]
    fn test_parse_row_normal() -> ErrStr<()> {
        let row = parse_test_row(&make_row("α", "14492", "yes", "yes"))?.unwrap();
        assert_eq!(row.name,    "α");
        assert_eq!(row.amount,  14492.0);
        assert_eq!(row.primary, "BTC");
        assert_eq!(row.pivot,   "UNDEAD");
        assert_eq!(row.pivots,  "15");
        assert_eq!(row.url,     "https://x.com/pivocateur/status/2069591552733712719");
        assert!(row.send);
        assert!(row.flipped);
        Ok(())
    }

    #[test]
    fn test_parse_row_send_no() -> ErrStr<()> {
        let row = parse_test_row(&make_row("τ", "2004", "no", "yes"))?.unwrap();
        assert!(!row.send, "send=no should parse as false");
        Ok(())
    }

    #[test]
    fn test_parse_row_flipped_no() -> ErrStr<()> {
        let row = parse_test_row(&make_row("γ", "42910", "yes", "no"))?.unwrap();
        assert!(!row.flipped, "flipped=no should parse as false");
        Ok(())
    }

    #[test]
    fn test_parse_row_amount_zero_skipped() -> ErrStr<()> {
        assert!(
            parse_test_row(&make_row("σ", "0", "yes", "yes"))?.is_none(),
            "amount=0 row should be skipped"
        );
        Ok(())
    }

    #[test]
    fn test_parse_row_blank_skipped() -> ErrStr<()> {
        assert!(parse_test_row("")?.is_none(),   "blank line should be skipped");
        assert!(parse_test_row("  ")?.is_none(), "whitespace line should be skipped");
        Ok(())
    }

    // ---- build_message -----------------------------------------------------
    #[test]
    fn test_build_message_normal() -> ErrStr<()> {
        let row = make_investor("α", 14492.0, true, false);
        let msg = build_message(&row)?;
        assert!(msg.contains("BTC-on-UNDEAD"),         "trade direction");
        assert!(msg.contains("BTC+UNDEAD pivot pool"), "pool name");
        assert!(msg.contains("14492 BTC"),             "amount + reinvested token");
        Ok(())
    }

    #[test]
    fn test_build_message_flipped() -> ErrStr<()> {
        // flipped only swaps which token is reinvested — trade direction
        // (the pool's actual X-on-Y order) never changes.
        let row = make_investor("α", 14492.0, true, true);
        let msg = build_message(&row)?;
        assert!(msg.contains("BTC-on-UNDEAD"),         "trade direction never flips");
        assert!(msg.contains("BTC+UNDEAD pivot pool"), "pool always prim+piv");
        assert!(msg.contains("14492 UNDEAD"),          "reinvested token is piv when flipped");
        Ok(())
    }

    #[test]
    fn test_build_message_singular_pivot() -> ErrStr<()> {
        let mut row = make_investor("α", 500.0, true, false);
        row.pivots = "1".to_string();
        let msg = build_message(&row)?;
        assert!(msg.contains("BTC-on-UNDEAD pivot "), "singular: no trailing 's'");
        Ok(())
    }

    #[test]
    fn test_build_message_plural_pivots() -> ErrStr<()> {
        // flipped=true here on purpose, to confirm trade direction stays
        // fixed even when flipped — only the reinvested token would swap.
        let msg = build_message(&make_investor("φ", 173748.0, true, true))?;
        assert!(msg.contains("15 BTC-on-UNDEAD pivots"), "plural pivot count, trade direction unaffected by flipped");
        Ok(())
    }

    #[test]
    fn test_build_message_exact_normal() -> ErrStr<()> {
        let mut row = make_investor("α", 1552.0, true, false);
        row.primary = "UNDEAD".to_string();
        row.pivot   = "USDC".to_string();
        row.pivots  = "1".to_string();
        row.url     = "https://x.com/pivocateur/status/2056884438156398786".to_string();
        assert_eq!(
            build_message(&row)?,
            "I close UNDEAD-on-USDC pivot (see tweet: \
             https://x.com/pivocateur/status/2056884438156398786). \
             I reinvest 1552 UNDEAD into the UNDEAD+USDC pivot pool for you."
        );
        Ok(())
    }

    #[test]
    fn test_parse_row_amount_invalid_errors() -> ErrStr<()> {
        // amount_reinvested deserializes straight into CommaFloat, so a bad
        // value fails at the csv/serde layer before parse_row ever runs —
        // this asserts on that actual error, not a custom message we don't produce.
        let err = parse_test_row(&make_row("ψ", "not-a-number", "yes", "yes"))
            .unwrap_err();
        assert!(err.contains("invalid float literal"), "should error loudly, not skip: {err}");
        Ok(())
    }

    #[test]
    fn test_parse_row_unrecognized_send_errors() -> ErrStr<()> {
        let err = parse_test_row(&make_row("α", "14492", "maybe", "yes")).unwrap_err();
        assert!(err.contains("send"), "should mention the field name");
        assert!(err.contains("maybe"), "should show the bad value");
        assert!(err.contains("yes/no/true/false"), "should show allowed values");
        Ok(())
    }

    #[test]
    fn test_parse_row_unrecognized_flipped_errors() -> ErrStr<()> {
        let err = parse_test_row(&make_row("α", "14492", "yes", "perhaps")).unwrap_err();
        assert!(err.contains("flipped"), "unrecognized flipped must error: {err}");
        Ok(())
    }

    #[test]
    fn test_parse_row_short_row_skipped() -> ErrStr<()> {
        // 12 columns — tx_url omitted (the malformed-export case)
        let short = "α\t100%\t3.46%\t14492\t0\tBTC\tUNDEAD\t$12.04\t15\t\
                     https://x.com/pivocateur/status/2069591552733712719\tyes\tyes";
        assert!(parse_test_row(short)?.is_none(), "a 12-column row must be skipped");
        Ok(())
    }

    #[test]
    fn test_build_message_small_decimal_amount() -> ErrStr<()> {
        // guards decimal-amount formatting (e.g. 0.002 BTC), not token choice —
        // previously named test_build_message_with_btc, which misdescribed its purpose
        let mut row = make_investor("α", 0.002, true, false);
        row.primary = "BTC".to_string();
        row.pivot   = "UNDEAD".to_string();
        row.pivots  = "1".to_string();
        row.url     = "https://x.com/pivocateur/status/2056884438156398786".to_string();
        assert_eq!(
            build_message(&row)?,
            "I close BTC-on-UNDEAD pivot (see tweet: \
            https://x.com/pivocateur/status/2056884438156398786). \
            I reinvest 0.002 BTC into the BTC+UNDEAD pivot pool for you."
        );
        Ok(())
    }
}

//============================================================================
//----- FUNCTIONAL TESTS -----------------------------------------------------
//============================================================================
#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
    use super::*;
    use paste::paste;
    use book::{create_testing, utils::now};
    use libs::investor_rows::test_functions::SendSpy;


    create_testing!("quiz11::a_reinvested");

    run!("processes_rows_without_sending", {
        // exercises the global_send: false path — what runoff_with_args hits
        // when the user answers "no" — distinct from the send-path test below
        // cols: 0=name 1=reinvested% 2=precentage 3=amount_reinvested 4=amount_distributed
        //       5=primary 6=pivot 7=usd 8=pivots 9=tweet_url 10=tx_url 11=send 12=flipped
        let tsv = "name\treinvested %\tprecentage\tamount reinvested\tamount distributed\t\
                   primary\tpivot\tUSD-value\tnumber of pivots closed\ttweet url\ttx url\tsend?\tflipped\n\
                   α\t100%\t3.46%\t14492\t0\tBTC\tUNDEAD\t$12.04\t15\t\
                   https://x.com/pivocateur/status/2069591552733712719\t\
                   https://snowtrace.io/tx/0xabc\tyes\tyes\n\
                   σ\t0%\t0.00%\t0\t0\tBTC\tUNDEAD\t$0.00\t15\t\
                   https://x.com/pivocateur/status/2069591552733712719\t\
                   https://snowtrace.io/tx/0xdef\tyes\tyes\n";

        let path_buf = std::env::temp_dir().join("reinvested_test.tsv");
        let path = path_buf.to_str().ok_or("temp path is not valid UTF-8")?;
        std::fs::write(path, tsv).map_err(|e| e.to_string())?;

        let spy = SendSpy::new();
        let spy_for_process = spy.clone();
        let _ = now(process_tsv(path, false, move |tok, id, txt| {
            Box::pin(spy_for_process.record(tok, id, txt))
        }))?;
    });

    // Fills the gap flagged in distributed's tests: confirms a skipped row
    // (amount reinvested == 0) genuinely never reaches send_fn, and that a
    // real row sends exactly once — rather than just eyeballing stdout.
    run!("skips_send_for_zero_amount_row", {
        unsafe {
            std::env::set_var("REINVESTED_BOT", "test-token");
            std::env::set_var("INVESTOR_CHAT_IDS", r#"{"α":11111}"#);
        }

        let tsv = "name\treinvested %\tprecentage\tamount reinvested\tamount distributed\t\
                   primary\tpivot\tUSD-value\tnumber of pivots closed\ttweet url\ttx url\tsend?\tflipped\n\
                   α\t100%\t3.46%\t14492\t0\tBTC\tUNDEAD\t$12.04\t15\t\
                   https://x.com/pivocateur/status/2069591552733712719\t\
                   https://snowtrace.io/tx/0xabc\tyes\tyes\n\
                   σ\t0%\t0.00%\t0\t0\tBTC\tUNDEAD\t$0.00\t15\t\
                   https://x.com/pivocateur/status/2069591552733712719\t\
                   https://snowtrace.io/tx/0xdef\tyes\tyes\n";

        let path_buf = std::env::temp_dir().join("reinvested_send_count_test.tsv");
        let path = path_buf.to_str().ok_or("temp path is not valid UTF-8")?;
        std::fs::write(path, tsv).map_err(|e| e.to_string())?;

        let spy = SendSpy::new();
        let spy_for_process = spy.clone();
        let _ = now(process_tsv(path, true, move |tok, id, txt| {
            Box::pin(spy_for_process.record(tok, id, txt))
        }))?;

        if spy.count() != 1 {
            return Err(format!(
                "expected exactly 1 send (σ's zero-amount row must be skipped), got {}",
                spy.count()
            ));
        }
        if !spy.sent_to(11111) {
            return Err("expected the one send to go to α's chat id".to_string());
        }
    });
}

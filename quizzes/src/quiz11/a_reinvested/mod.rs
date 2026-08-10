use clap::Parser;
use book::{
        parse_args_add_banner,
        cli_utils::generate_banner,
        err_utils::ErrStr,
        string_utils::plural,
        utils::get_env,
};
use libs::{
        investor_rows::{
            InvestorRow,
            chat_id_for,
            deserialize_row,
            send_telegram,
            SendFuture
        }
};

//============================================================================
//----- Message Building -------------------------------------------------
//============================================================================
pub fn build_message(row: &InvestorRow) -> ErrStr<String> {
    let prim = &row.primary;
    let piv  = &row.pivot;
    let pool = format!("{prim}+{piv}");
    let trade = format!("{prim}-on-{piv}");
    let reinvested = if row.flipped { piv.as_str() } else { prim.as_str() };
    let n      = row.pivots;
    let noun   = format!("{trade} pivot");
    let pivots = plural(n, &noun);
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
// File-reading step (read + skip header) now lives in
// libs::investor_rows::tsv_data_rows — identical to distributed's, no
// reason to hand-roll it twice. The per-row async loop stays local since
// mb_send_row genuinely differs between the two binaries.
pub async fn process_tsv<F>(tsv_path: &str, global_send: bool, send_fn: F)
   -> ErrStr<()> where F: for<'a> Fn(&'a str, i64, &'a str) -> SendFuture<'a> {
    let rows = libs::investor_rows::tsv_data_rows(tsv_path)?;
    for line in rows { mb_send_row(&line, global_send, &send_fn).await?; }
    Ok(())
}

async fn mb_send_row<F>(line: &str, global_send: bool, send_fn: &F)
   -> ErrStr<()> where F: for<'a> Fn(&'a str, i64, &'a str) -> SendFuture<'a> {
    let mb_row = deserialize_row::<InvestorRow>(&line)?;
    if let Some(row) = mb_row {
       let msg = build_message(&row)?;
       println!("[{}] {msg}", row.name);

       let amt: f32 = row.amount.into();
       if global_send && row.send && amt > 0.0 {
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
#[command(version = "2.06")]
struct Args {
   /// The path to the list of the investors and their distributions
   tsv_path: String,

   /// Send a telegram? (yes/no)
   #[arg(short, long)]
   send: bool
}

pub async fn runoff_with_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   process_tsv(&args.tsv_path, args.send, |tok, id, txt| {
                Box::pin(send_telegram(tok, id, txt))
   }).await
}

//============================================================================
//----- UNIT TESTS -----------------------------------------------------------
//============================================================================
#[cfg(test)]
mod unit_tests {
    use super::*;
    use libs::investor_rows::{ deserialize_row, spies::SendSpy };
    use book::num::floats::comma_floats::CommaFloat;

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
            amount: CommaFloat(amount),
            primary: "BTC".to_string(),
            pivot:   "UNDEAD".to_string(),
            pivots:  15,
            url:     "https://x.com/pivocateur".to_string(),
            send,
            flipped,
        }
    }

    fn parse_test_row(line: &str) -> ErrStr<Option<InvestorRow>> {
       deserialize_row(line)
    }

    // ---- parse_row ---------------------------------------------------------
    #[test]
    fn test_parse_row_normal() -> ErrStr<()> {
        let row = parse_test_row(&make_row("α", "14492", "yes", "yes"))?.unwrap();
        assert_eq!(row.name,    "α");
        let amt: f32 = row.amount.into();
        assert_eq!(amt, 14492.0);
        assert_eq!(row.primary, "BTC");
        assert_eq!(row.pivot,   "UNDEAD");
        assert_eq!(row.pivots,  15);
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

    #[tokio::test]
    async fn test_parse_row_amount_zero_skipped() -> ErrStr<()> {
        let spy = SendSpy::new();
            let spy_for_process = spy.clone();
       let line = make_row("σ", "0", "yes", "yes");
       mb_send_row(&line, true, &move |tok, id, txt| {
            Box::pin(spy_for_process.record(tok, id, txt, false))
       }).await?;
       assert_eq!(0, spy.count(),
             "Should not have sent a message with no amount {line}");
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
        row.pivots = 1;
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
        row.pivots  = 1;
        row.url     = "https://x.com/pivocateur/status/2056884438156398786".to_string();
        let msg = build_message(&row)?;
        assert!(msg.contains("UNDEAD-on-USDC"), "message wrong way pivot {msg}");
        assert!(msg.contains("1552 UNDEAD"), "message returning wrong amount or wrong token {msg}");
        assert!(msg.contains("status/20568844"), "message has wrong tweet {msg}");
        Ok(())
    }

    #[test]
    fn fail_parse_row_amount_invalid_errors() -> ErrStr<()> {
        // amount_reinvested deserializes straight into CommaFloat, so a bad
        // value fails at the csv/serde layer before parse_row ever runs —
        // this asserts on that actual error, not a custom message we don't produce.
        let err = parse_test_row(&make_row("ψ", "not-a-number", "yes", "yes"))
            .unwrap_err();
        assert!(err.contains("invalid float literal"), "should error loudly, not skip: {err}");
        Ok(())
    }

    #[test]
    fn fail_parse_row_unrecognized_flipped_errors() -> ErrStr<()> {
        let err =
         parse_test_row(&make_row("α", "14492", "yes", "perhaps")).unwrap_err();
        assert!(err.contains("Unable to parse BoolCell from perhaps"),
                "unrecognized flipped must error: {err}");
        Ok(())
    }

    #[test]
    fn test_build_message_small_decimal_amount() -> ErrStr<()> {
        // guards decimal-amount formatting (e.g. 0.002 BTC), not token choice —
        // previously named test_build_message_with_btc, which misdescribed its purpose
        let mut row = make_investor("α", 0.002, true, false);
        row.primary = "BTC".to_string();
        row.pivot   = "UNDEAD".to_string();
        row.pivots  = 1;
        row.url     = "https://x.com/pivocateur/status/2056884438156398786".to_string();
        let msg =  build_message(&row)?;
        assert!(msg.contains("BTC-on-UNDEAD"), "message contains wrong way pivot {msg}");
        assert!(msg.contains("0.002 BTC"), "message doesn't have little BTC {msg}");
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
    use libs::investor_rows::{ spies::SendSpy, INVESTOR_TSV_HEADER};

    create_testing!("quiz11::a_reinvested");

    run!("processes_rows_without_sending", {
        // exercises the global_send: false path — what runoff_with_args hits
        // when the user answers "no" — distinct from the send-path test below
        // cols: 0=name 1=reinvested% 2=precentage 3=amount_reinvested 4=amount_distributed
        //       5=primary 6=pivot 7=usd 8=pivots 9=tweet_url 10=tx_url 11=send 12=flipped
        let tsv = format!(
            "{INVESTOR_TSV_HEADER}\n\
             α\t100%\t3.46%\t14492\t0\tBTC\tUNDEAD\t$12.04\t15\t\
             https://x.com/pivocateur/status/2069591552733712719\t\
             https://snowtrace.io/tx/0xabc\tyes\tyes\n\
             σ\t0%\t0.00%\t0\t0\tBTC\tUNDEAD\t$0.00\t15\t\
             https://x.com/pivocateur/status/2069591552733712719\t\
             https://snowtrace.io/tx/0xdef\tyes\tyes\n"
        );

        let path_buf = std::env::temp_dir().join("reinvested_test.tsv");
        let path = path_buf.to_str().ok_or("temp path is not valid UTF-8")?;
        std::fs::write(path, tsv).map_err(|e| e.to_string())?;

        let spy = SendSpy::new();
        let spy_for_process = spy.clone();
        let _ = now(process_tsv(path, false, move |tok, id, txt| {
            Box::pin(spy_for_process.record(tok, id, txt, false))
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

        let tsv = format!(
            "{INVESTOR_TSV_HEADER}\n\
             α\t100%\t3.46%\t14492\t0\tBTC\tUNDEAD\t$12.04\t15\t\
             https://x.com/pivocateur/status/2069591552733712719\t\
             https://snowtrace.io/tx/0xabc\tyes\tyes\n\
             σ\t0%\t0.00%\t0\t0\tBTC\tUNDEAD\t$0.00\t15\t\
             https://x.com/pivocateur/status/2069591552733712719\t\
             https://snowtrace.io/tx/0xdef\tyes\tyes\n"
        );

        let path_buf = std::env::temp_dir().join("reinvested_send_count_test.tsv");
        let path = path_buf.to_str().ok_or("temp path is not valid UTF-8")?;
        std::fs::write(path, tsv).map_err(|e| e.to_string())?;

        let spy = SendSpy::new();
        let spy_for_process = spy.clone();
        let _ = now(process_tsv(path, true, move |tok, id, txt| {
            Box::pin(spy_for_process.record(tok, id, txt, false))
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

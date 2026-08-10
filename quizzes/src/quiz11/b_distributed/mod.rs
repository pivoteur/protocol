use clap::Parser;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use book::{
        parse_args_add_banner,
        cli_utils::generate_banner,
        err_utils::ErrStr,
        utils::get_env,
        num::floats::comma_floats::CommaFloat
};
use libs::{
        investor_rows::{
            chat_id_for,
            deserialize_row,
            deserialize_yes_no_bool,
            send_telegram,
            SendFuture
        }
};

//============================================================================
//----- CSV Row Parsing ------------------------------------------------------
//============================================================================
#[derive(Debug)]
pub struct DistributionRow {
    pub name:    String,
    pub amount:  f32,
    pub primary: String,
    pub pivot:   String,
    pub url:     String,  // tweet url
    pub tx_url:  String,
    pub send:    bool,
    pub flipped: bool,
}

#[serde_as]
#[derive(Debug, Deserialize)]
struct DistributionRecord {
    name: String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "amount distributed")]
    amount_distributed: CommaFloat,
    primary: String,
    pivot: String,
    #[serde(rename = "tweet url")]
    tweet_url: String,
    #[serde(rename = "tx url")]
    tx_url: String,
    #[serde(rename = "send?")]
    #[serde(deserialize_with = "deserialize_yes_no_bool")]
    send: bool,
    #[serde(deserialize_with = "deserialize_yes_no_bool")]
    flipped: bool
}

/// Returns `Ok(None)` only for rows where amount distributed == 0 (handled
/// by `reinvested`, not `distributed`). Returns `Err` for malformed data
fn parse_row(record: &DistributionRecord) -> ErrStr<Option<DistributionRow>> {
    let name    = record.name.trim();
    let amount: f32 = record.amount_distributed.into();
    let primary = record.primary.trim();
    let pivot   = record.pivot.trim();
    let url     = record.tweet_url.trim();
    let tx_url  = record.tx_url.trim();

    if amount == 0.0 {
        return Ok(None);
    }

    Ok(Some(DistributionRow {
        name:    name.to_string(),
        amount,
        primary: primary.to_string(),
        pivot:   pivot.to_string(),
        url:     url.to_string(),
        tx_url:  tx_url.to_string(),
        send: record.send,
        flipped: record.flipped
    }))
}

//============================================================================
//----- Message Building -------------------------------------------------
//============================================================================
pub fn build_message(row: &DistributionRow) -> String {
    let prim = &row.primary;
    let piv  = &row.pivot;
    let trade = format!("{prim}-on-{piv}");
    let sent_token = if row.flipped { piv.as_str() } else { prim.as_str() };
    let ans = format!(
        "I close an {trade} pivot (please see the twitter post: {tweet_url}). \
         I sent {amount} {sent_token} to you; tx_id: {tx_url}",
        tweet_url = row.url,
        amount    = row.amount,
        tx_url    = row.tx_url,
    );
    ans
}

//============================================================================
//----- Core: process all rows in one pass -----------------------------------
//============================================================================
// File-reading step (read + skip header) now lives in
// libs::investor_rows::tsv_data_rows — identical to reinvested's, no
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
    let Some(record) = deserialize_row::<DistributionRecord>(line)? else { return Ok(()) };
    let Some(row) = parse_row(&record)? else { return Ok(()) };

    let msg = build_message(&row);
    println!("[{}] {msg}", row.name);

    if global_send && row.send {
        let bot_token = get_env("REINVESTED_BOT")?;
        let chat_id   = chat_id_for(&row.name)?;
        send_fn(&bot_token, chat_id, &msg).await?;
    }
    Ok(())
}

//============================================================================
//----- fn runoff_with_args --------------------------------------------------
//============================================================================
/// Sends distribution message to investors
/// The investors and their distributions are listed in TSV file
#[derive(Debug, Parser)]
#[command(name = "distributed")]
#[command(version = "2.05")]
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

    // ---- helpers -----------------------------------------------------------
    fn make_row(name: &str, amount: &str, send: &str, flipped: &str) -> String {
        // cols: 0=name 1=reinvested% 2=precentage 3=amount_reinvested 4=amount_distributed
        //       5=primary 6=pivot 7=usd 8=pivots 9=tweet_url 10=tx_url 11=send 12=flipped
        let ans = format!(
            "{name}\t0%\t10.25%\t0\t{amount}\tBTC\tUNDEAD\t$35.66\t15\t\
             https://x.com/pivocateur/status/2069591552733712719\t\
             https://snowtrace.io/tx/0x04454ba7f8484359d821f18a5c5e1e6334fa43c416ec345d1de6df10c3e13765\t\
             {send}\t{flipped}"
        );
        ans
    }

    fn make_distribution(name: &str, amount: f32, send: bool, flipped: bool) -> DistributionRow {
        DistributionRow {
            name:    name.to_string(),
            amount,
            primary: "BTC".to_string(),
            pivot:   "UNDEAD".to_string(),
            url:     "https://x.com/pivocateur".to_string(),
            tx_url:  "https://snowtrace.io/tx/0xabc".to_string(),
            send,
            flipped,
        }
    }

    // Delegates the raw TSV -> DistributionRecord step to the shared
    // `deserialize_row` helper in libs::investor_rows (same one `reinvested`
    // uses for InvestorRow) — only parse_row stays local, since
    // DistributionRow/DistributionRecord are distributed-only.
    fn parse_test_row(line: &str) -> ErrStr<Option<DistributionRow>> {
        match deserialize_row::<DistributionRecord>(line)? {
            None         => Ok(None),
            Some(record) => parse_row(&record),
        }
    }

    // ---- parse_row ---------------------------------------------------------
    #[test]
    fn test_parse_row_normal() -> ErrStr<()> {
        let row = parse_test_row(&make_row("γ", "42910", "yes", "yes"))?.unwrap();
        assert_eq!(row.name,    "γ");
        assert_eq!(row.amount,  42910.0);
        assert_eq!(row.primary, "BTC");
        assert_eq!(row.pivot,   "UNDEAD");
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
            parse_test_row(&make_row("α", "0", "yes", "yes"))?.is_none(),
            "amount distributed=0 row should be skipped"
        );
        Ok(())
    }

    #[test]
    fn test_parse_row_blank_skipped() -> ErrStr<()> {
        assert!(parse_test_row("")?.is_none(),   "blank line should be skipped");
        assert!(parse_test_row("  ")?.is_none(), "whitespace line should be skipped");
        Ok(())
    }

    #[test]
    fn test_parse_row_amount_invalid_errors() -> ErrStr<()> {
        // amount_distributed deserializes straight into CommaFloat, so a bad
        // value fails at the csv/serde layer before parse_row ever runs —
        // same shape as reinvested's equivalent test.
        let err = parse_test_row(&make_row("ψ", "not-a-number", "yes", "yes"))
            .unwrap_err();
        assert!(err.contains("invalid float literal"), "should error loudly, not skip: {err}");
        Ok(())
    }

    // ---- build_message -----------------------------------------------------
    #[test]
    fn test_build_message_exact() {
        let mut row = make_distribution("γ", 4349.0, true, false);
        row.primary = "USDC".to_string();
        row.pivot   = "UNDEAD".to_string();
        row.url     = "x.com/pivocateur/status/2054570565474635869".to_string();
        row.tx_url  = "snowtrace.io/tx/0x04454ba7f8484359d821f18a5c5e1e6334fa43c416ec345d1de6df10c3e13765".to_string();
        let msg = build_message(&row);
        assert_eq!(
            msg,
            "I close an USDC-on-UNDEAD pivot \
             (please see the twitter post: x.com/pivocateur/status/2054570565474635869). \
             I sent 4349 USDC to you; \
             tx_id: snowtrace.io/tx/0x04454ba7f8484359d821f18a5c5e1e6334fa43c416ec345d1de6df10c3e13765"
        );
    }

    #[test]
    fn test_build_message_flipped() {
        // flipped only swaps which token was sent — trade direction (the
        // pool's actual X-on-Y order) never changes, per the domain rule.
        let row = make_distribution("γ", 4349.0, true, true);
        let msg = build_message(&row);
        assert!(msg.contains("BTC-on-UNDEAD"), "trade direction never flips");
        assert!(msg.contains("4349 UNDEAD"),   "sent token is pivot when flipped");
    }

    #[test]
    fn test_build_message_tx_url_passthrough() {
        // tx_url isn't a token and isn't required to look like a real URL —
        // this confirms it's substituted verbatim, unmodified/untruncated,
        // distinct from test_build_message_exact's real long tx_url.
        let mut row = make_distribution("τ", 2004.0, true, false);
        row.tx_url = "tx".to_string();
        let msg = build_message(&row);
        assert!(
            msg.ends_with("tx_id: tx"),
            "tx_url must appear verbatim after tx_id:, not reformatted or truncated"
        );
    }

    #[test]
    fn test_build_message_different_token_pair() {
        let mut row = make_distribution("δ", 1851.0, true, false);
        row.primary = "AVAX".to_string();
        row.pivot   = "BTC".to_string();
        let msg = build_message(&row);
        assert!(msg.contains("AVAX-on-BTC"));
        assert!(msg.contains("1851 AVAX"));
    }

    #[test]
    fn test_parse_row_unrecognized_send_errors() -> ErrStr<()> {
        // send/flipped now go through the same shared `deserialize_yes_no_bool`
        // InvestorRow uses, so a bad cell fails the same way reinvested's does
        // (BoolCell-style message) instead of a generic csv/serde bool error.
        let err = parse_test_row(&make_row("γ", "42910", "maybe", "yes")).unwrap_err();
        assert!(err.contains("Unable to parse BoolCell from maybe"), "unrecognized send must error: {err}");
        Ok(())
    }

    #[test]
    fn test_parse_row_unrecognized_flipped_errors() -> ErrStr<()> {
        let err = parse_test_row(&make_row("γ", "42910", "yes", "perhaps")).unwrap_err();
        assert!(err.contains("Unable to parse BoolCell from perhaps"), "unrecognized flipped must error: {err}");
        Ok(())
    }

    #[test]
    fn test_parse_row_short_row_skipped() -> ErrStr<()> {
        // 12 columns — tx_url omitted (the malformed-export case)
        let short = "γ\t0%\t10.25%\t0\t42910\tBTC\tUNDEAD\t$35.66\t15\t\
                     https://x.com/pivocateur/status/2069591552733712719\tyes\tyes";
        assert!(parse_test_row(short)?.is_none(), "a 12-column row must be skipped");
        Ok(())
    }

    #[test]
    fn test_parse_row_reads_url_and_tx_url() -> ErrStr<()> {
        let row = parse_test_row(&make_row("γ", "42910", "yes", "yes"))?.unwrap();
        assert_eq!(row.url,    "https://x.com/pivocateur/status/2069591552733712719");
        assert_eq!(row.tx_url, "https://snowtrace.io/tx/0x04454ba7f8484359d821f18a5c5e1e6334fa43c416ec345d1de6df10c3e13765");
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


    create_testing!("quiz11::b_distributed");

    run!("processes_rows_without_sending", {
        // col: 0=name 1=reinvested% 2=precentage 3=amount_reinvested 4=amount_distributed
        //      5=primary 6=pivot 7=usd 8=pivots 9=tweet_url 10=tx_url 11=send 12=flipped
        let tsv = format!(
            "{INVESTOR_TSV_HEADER}\n\
             γ\t0%\t10.25%\t0\t42910\tBTC\tUNDEAD\t$35.66\t15\t\
             https://x.com/pivocateur/status/2069591552733712719\t\
             https://snowtrace.io/tx/0xabc\tyes\tyes\n\
             α\t100%\t3.46%\t14492\t0\tBTC\tUNDEAD\t$12.04\t15\t\
             https://x.com/pivocateur/status/2069591552733712719\t\
             https://snowtrace.io/tx/0xdef\tyes\tyes\n"
        );

        let path_buf = std::env::temp_dir().join("distributed_test.tsv");
        let path = path_buf.to_str().ok_or("temp path is not valid UTF-8")?;
        std::fs::write(path, tsv).map_err(|e| e.to_string())?;

        let spy = SendSpy::new();
        let spy_for_process = spy.clone();
        let _ = now(process_tsv(path, false, move |tok, id, txt| {
            Box::pin(spy_for_process.record(tok, id, txt, false))
        }))?;
    });

    run!("skips_send_for_zero_amount_row", {
        unsafe {
            std::env::set_var("REINVESTED_BOT", "test-token");
            std::env::set_var("INVESTOR_CHAT_IDS", r#"{"γ":22222}"#);
        }

        let tsv = format!(
            "{INVESTOR_TSV_HEADER}\n\
             γ\t0%\t10.25%\t0\t42910\tBTC\tUNDEAD\t$35.66\t15\t\
             https://x.com/pivocateur/status/2069591552733712719\t\
             https://snowtrace.io/tx/0xabc\tyes\tyes\n\
             α\t100%\t3.46%\t14492\t0\tBTC\tUNDEAD\t$12.04\t15\t\
             https://x.com/pivocateur/status/2069591552733712719\t\
             https://snowtrace.io/tx/0xdef\tyes\tyes\n"
        );

        let path_buf = std::env::temp_dir().join("distributed_send_count_test.tsv");
        let path = path_buf.to_str().ok_or("temp path is not valid UTF-8")?;
        std::fs::write(path, tsv).map_err(|e| e.to_string())?;

        let spy = SendSpy::new();
        let spy_for_process = spy.clone();
        let _ = now(process_tsv(path, true, move |tok, id, txt| {
            Box::pin(spy_for_process.record(tok, id, txt, false))
        }))?;

        if spy.count() != 1 {
            return Err(format!(
                "expected exactly 1 send (α's zero-amount-distributed row must be skipped), got {}",
                spy.count()
            ));
        }
        if !spy.sent_to(22222) {
            return Err("expected the one send to go to γ's chat id".to_string());
        }
    });
}

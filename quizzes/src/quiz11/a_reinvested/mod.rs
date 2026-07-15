use std::pin::Pin;
use clap::Parser;
use reqwest::Client;
use csv::{ReaderBuilder, ErrorKind, DeserializeErrorKind};
use serde::Deserialize;
use book::{
    parse_args_add_banner,
    cli_utils::add_banner,
    err_utils::{ ErrStr, err_or },
    parse_utils::parse_id,
    string_utils::plural,
    utils::get_env
};

//============================================================================
//----- Telegram Configuration -----------------------------------------------
//============================================================================
fn chat_id_for(investor: &str) -> ErrStr<i64> {
    let raw = get_env("INVESTOR_CHAT_IDS")?;
    let map: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("INVESTOR_CHAT_IDS is not valid JSON: {e}"))?;
    map.get(investor)
        .and_then(|v| v.as_i64())
        .ok_or_else(|| format!("unknown investor/ chat id doesn't exist: {investor}"))
}

//============================================================================
//----- CSV Row Parsing ------------------------------------------------------
//============================================================================
#[derive(Debug)]
pub struct InvestorRow {
    pub name:    String,
    pub amount:  u64,
    pub primary: String,
    pub pivot:   String,
    pub pivots:  String,
    pub url:     String,
    pub send:    bool,
    pub flipped: bool,
}

#[derive(Debug, Deserialize)]
struct PivotRecord {
    name: String,
    #[serde(rename = "reinvested %")]
    _reinvested_pct: String,
    #[serde(rename = "precentage")]
    _percentage: String,
    #[serde(rename = "amount reinvested")]
    amount_reinvested: String,
    #[serde(rename = "amount distributed")]
    _amount_distributed: String,
    primary: String,
    pivot: String,
    #[serde(rename = "USD-value")]
    _usd_value: String,
    #[serde(rename = "number of pivots closed")]
    pivots: String,
    #[serde(rename = "tweet url")]
    tweet_url: String,
    #[serde(rename = "tx url")]
    _tx_url: String,
    #[serde(rename = "send?")]
    send: String,
    flipped: String,
}

fn parse_bool_cell(field: &str, raw: &str) -> ErrStr<bool> {
    match raw.trim().to_lowercase().as_str() {
        "yes" | "true"  => Ok(true),
        "no"  | "false" => Ok(false),
        other => Err(format!("column '{field}': unrecognized value '{other}'. Expected yes/no/true/false.")),
    }
}
/// Returns `Ok(None)` only for rows where amount reinvested == 0 (handled
/// by `distributed`, not `reinvested`). Returns `Err` for malformed data.
/// Structural issues (blank lines, ragged/short rows, the header row) never
/// reach this function — the CSV reader in `process_csv` filters them out.
fn parse_row(record: &PivotRecord) -> ErrStr<Option<InvestorRow>> {
    let name    = record.name.trim();
    let amount  = record.amount_reinvested.trim();
    let primary = record.primary.trim();
    let pivot   = record.pivot.trim();
    let pivots  = record.pivots.trim();
    let url     = record.tweet_url.trim();

    let amount_val: u64 = match amount.parse() {
        Ok(v) => v,
        Err(e) => return Err(format!(
            "row '{name}': invalid amount reinvested '{amount}': {e}"
        )),
    };
    if amount_val == 0 {
        return Ok(None);
    }

    let send    = parse_bool_cell("send", &record.send)?;
    let flipped = parse_bool_cell("flipped", &record.flipped)?;

    Ok(Some(InvestorRow {
        name:    name.to_string(),
        amount:  amount_val,
        primary: primary.to_string(),
        pivot:   pivot.to_string(),
        pivots:  pivots.to_string(),
        url:     url.to_string(),
        send,
        flipped,
    }))
}

fn is_ragged_row(e: &csv::Error) -> bool {
    matches!(
        e.kind(),
        ErrorKind::Deserialize { err, .. } if matches!(err.kind(), DeserializeErrorKind::UnexpectedEndOfRow)
    )
}

//============================================================================
//----- Message Building and Sending -----------------------------------------
//============================================================================
pub fn build_message(row: &InvestorRow) -> ErrStr<String> {
    let prim = &row.primary;
    let piv  = &row.pivot;
    let pool = format!("{prim}+{piv}");
    let (reinvested, trade) = if row.flipped {
        (piv.as_str(), format!("{piv}-on-{prim}"))
    } else {
        (prim.as_str(), format!("{prim}-on-{piv}"))
    };
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

pub async fn send_telegram(bot_token: &str, chat_id: i64, text: &str) -> ErrStr<()> {
    let url = format!("https://api.telegram.org/bot{bot_token}/sendMessage");
    Client::new()
        .post(&url)
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text":    text,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
pub async fn mock_send_telegram(_bot_token: &str, chat_id: i64, text: &str) -> ErrStr<()> {
    println!("[mock telegram] chat_id={chat_id} | text={text}");
    Ok(())
}

//============================================================================
//----- Core: process all rows in one pass -----------------------------------
//============================================================================
type SendFuture<'a> = Pin<Box<dyn std::future::Future<Output = ErrStr<()>> + Send + 'a>>;

pub async fn process_csv<F>(csv_path: &str, global_send: bool, send_fn: F)
   -> ErrStr<()> where F: for<'a> Fn(&'a str, i64, &'a str) -> SendFuture<'a> {
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .from_path(csv_path)
        .map_err(|e| format!("cannot read '{csv_path}': {e}"))?;

    for result in rdr.deserialize::<PivotRecord>() {
        let record = match result {
            Ok(r) => r,
            Err(e) if is_ragged_row(&e) => continue,
            Err(e) => return Err(format!("malformed row in '{csv_path}': {e}")),
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
#[command(version = "1.02")]
struct Args {
   /// The path to the list of the investors and their distributions
   csv_path: String,

   /// Send a telegram? (yes/no)
   send: String
}

pub async fn runoff_with_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   let send: bool = err_or(args.send.parse(),
       &format!("Cannot parse {} into boolean-value", args.send))?;
   process_csv(&args.csv_path, send, |tok, id, txt| {
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
    fn make_row(
        name: &str, amount: &str, send: &str, flipped: &str,
    ) -> String {
        // cols: 0=name 1=reinvested% 2=precentage 3=amount_reinvested 4=amount_distributed
        //        5=primary 6=pivot 7=usd 8=pivots 9=tweet_url 10=tx_url 11=send 12=flipped
        format!(
            "{name}\t100%\t3.46%\t{amount}\t0\tBTC\tUNDEAD\t$12.04\t15\t\
             https://x.com/pivocateur/status/2069591552733712719\t\
             https://snowtrace.io/tx/0xabc\t{send}\t{flipped}"
        )
    }

    fn make_investor(name: &str, amount: u64, send: bool, flipped: bool) -> InvestorRow {
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

    fn parse_test_row(line: &str) -> ErrStr<Option<InvestorRow>> {
        let header = "name\treinvested %\tprecentage\tamount reinvested\tamount distributed\t\
                      primary\tpivot\tUSD-value\tnumber of pivots closed\ttweet url\ttx url\tsend?\tflipped";
        let tsv = format!("{header}\n{line}\n");
        let mut rdr = ReaderBuilder::new()
            .delimiter(b'\t')
            .flexible(true)
            .from_reader(tsv.as_bytes());

        match rdr.deserialize::<PivotRecord>().next() {
            None                              => Ok(None),
            Some(Err(e)) if is_ragged_row(&e) => Ok(None),
            Some(Err(e))                      => Err(format!("test fixture malformed: {e}")),
            Some(Ok(record))                  => parse_row(&record),
        }
    }

    // ---- parse_row ---------------------------------------------------------
    #[test]
    fn test_parse_row_normal() -> ErrStr<()> {
        let row = parse_test_row(&make_row("α", "14492", "yes", "yes"))?.unwrap();
        assert_eq!(row.name,    "α");
        assert_eq!(row.amount,  14492);
        assert_eq!(row.primary, "BTC");
        assert_eq!(row.pivot,   "UNDEAD");
        assert_eq!(row.pivots,  "15");
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
    fn test_parse_row_column_header_repeated_errors() -> ErrStr<()> {
        let header = "name\treinvested %\tprecentage\tamount reinvested\tamount distributed\t\
                      primary\tpivot\tUSD-value\tnumber of pivots closed\ttweet url\ttx url\tsend?\tflipped";
        let err = parse_test_row(header).unwrap_err();
        assert!(err.contains("invalid amount reinvested"), "a duplicated header row should now error loudly, not skip silently: {err}");
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
        let row = make_investor("α", 14492, true, false);
        let msg = build_message(&row)?;
        assert!(msg.contains("BTC-on-UNDEAD"),         "trade direction");
        assert!(msg.contains("BTC+UNDEAD pivot pool"), "pool name");
        assert!(msg.contains("14492 BTC"),             "amount + reinvested token");
        Ok(())
    }

    #[test]
    fn test_build_message_flipped() -> ErrStr<()> {
        let row = make_investor("α", 14492, true, true);
        let msg = build_message(&row)?;
        assert!(msg.contains("UNDEAD-on-BTC"),         "flipped trade direction");
        assert!(msg.contains("BTC+UNDEAD pivot pool"), "pool always prim+piv");
        assert!(msg.contains("14492 UNDEAD"),          "reinvested token is piv when flipped");
        Ok(())
    }

    #[test]
    fn test_build_message_singular_pivot() -> ErrStr<()> {
        let mut row = make_investor("α", 500, true, false);
        row.pivots = "1".to_string();
        let msg = build_message(&row)?;
        assert!(msg.contains("BTC-on-UNDEAD pivot "), "singular: no trailing 's'");
        Ok(())
    }

    #[test]
    fn test_build_message_plural_pivots() -> ErrStr<()> {
        let msg = build_message(&make_investor("φ", 173748, true, true))?;
        assert!(msg.contains("15 UNDEAD-on-BTC pivots"), "plural pivot count");
        Ok(())
    }

    #[test]
    fn test_build_message_exact_normal() -> ErrStr<()> {
        let mut row = make_investor("α", 1552, true, false);
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
    fn test_build_message_exact_flipped() -> ErrStr<()> {
        let mut row = make_investor("α", 500, true, true);
        row.pivots = "1".to_string();
        row.url    = "https://x.com/pivocateur/status/2056884438156398786".to_string();
        assert_eq!(
            build_message(&row)?,
            "I close UNDEAD-on-BTC pivot (see tweet: \
             https://x.com/pivocateur/status/2056884438156398786). \
             I reinvest 500 UNDEAD into the BTC+UNDEAD pivot pool for you."
        );
        Ok(())
    }

    #[test]
    fn test_parse_row_amount_invalid_errors() -> ErrStr<()> {
        let err = parse_test_row(&make_row("ψ", "not-a-number", "yes", "yes"))
            .unwrap_err();
        assert!(err.contains("invalid amount"), "should error loudly, not skip");
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
    fn test_parse_row_reads_tweet_url() -> ErrStr<()> {
        let row = parse_test_row(&make_row("α", "14492", "yes", "yes"))?.unwrap();
        assert_eq!(row.url, "https://x.com/pivocateur/status/2069591552733712719");
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
    use book::{ create_testing, utils::now };

    
    create_testing!("quiz11::a_reinvested");

    run!("mock_process_csv", {
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

        let _ = now(process_csv(path, false, |tok, id, txt| {
            Box::pin(mock_send_telegram(tok, id, txt))
        }))?;
    });
}

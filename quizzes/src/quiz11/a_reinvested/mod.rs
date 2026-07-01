use std::pin::Pin;
use reqwest::Client;
use book::{
    parse_utils::parse_id,
    string_utils::plural,
    err_utils::ErrStr,
    utils::{ 
        get_args, 
        get_env 
    },
};


//============================================================================
//----- Version/ App_Name/ Usage ---------------------------------------------
//============================================================================
fn version()  -> &'static str { "2.00" }
fn app_name() -> &'static str { "reinvested" }

fn usage() -> ErrStr<()> {
    eprintln!("Usage: {} <csv_path> <send>", app_name());
    eprintln!("  csv_path : path to the shared investors TSV file");
    eprintln!("  send     : let Robbie send messages? (true/false)");
    eprintln!();
    eprintln!("TSV columns:");
    eprintln!("  name | reinvested % | precentage | amount reinvested | amount distributed");
    eprintln!("  primary | pivot | USD-value | number of pivots closed | tweet url | tx url | send? | flipped");
    Err("Need <csv_path> <send> arguments".to_string())
}

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

/// Returns `Ok(None)` for rows that should be skipped:
///   - blank lines
///   - the column-name header row              (col 0 == "name")
///   - data rows where amount reinvested == 0  (handled by `distributed`, not `reinvested`)
/// Returns `Err` only for rows that look like data but are malformed.
pub fn parse_row(line: &str) -> ErrStr<Option<InvestorRow>> {
    let line = line.trim();
    if line.is_empty() {
        return Ok(None);
    }

    let cols: Vec<&str> = line.split('\t').collect();

    // not enough columns to be a data row
    if cols.len() < 13 {
        return Ok(None);
    }

    // col: 0=name 1=reinvested% 2=precentage 3=amount_reinvested 4=amount_distributed
    //      5=primary 6=pivot 7=usd 8=pivots 9=tweet_url 10=tx_url 11=send 12=flipped
    let name    = cols[0].trim();
    let amount  = cols[3].trim();
    let primary = cols[5].trim();
    let pivot   = cols[6].trim();
    let pivots  = cols[8].trim();
    let url     = cols[9].trim();
    let send_s  = cols[11].trim().to_lowercase();
    let flip_s  = cols[12].trim().to_lowercase();

    // column-name header row
    if name == "name" {
        return Ok(None);
    }

    // skip investors with nothing reinvested this run
    let amount_val: u64 = match amount.parse() {
        Ok(v) => v,
        Err(e) => return Err(format!(
            "row '{name}': invalid amount reinvested '{amount}': {e}"
        )),
    };
    if amount_val == 0 {
        return Ok(None);
    }

    let send    = send_s == "yes" || send_s == "true";
    let flipped = flip_s == "yes" || flip_s == "true";

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

pub async fn process_csv<F>(
    csv_path:    &str,
    global_send: bool,
    send_fn:     F,
) -> ErrStr<()>
where
    F: for<'a> Fn(&'a str, i64, &'a str) -> SendFuture<'a>,
{
    let content = std::fs::read_to_string(csv_path)
        .map_err(|e| format!("cannot read '{csv_path}': {e}"))?;

    for line in content.lines() {
        let Some(row) = parse_row(line)? else { continue };

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

//============================================================================
//----- fn runoff_with_args --------------------------------------------------
//============================================================================
pub async fn runoff_with_args() -> ErrStr<()> {
    eprintln!("{}, version: {}", app_name(), version());
    let args = get_args();
    match args.as_slice() {
        [csv_path, send] => {
            let global_send = send.parse::<bool>()
                .map_err(|_| format!("send must be true or false, got: {send}"))?;
            process_csv(csv_path, global_send, |tok, id, txt| {
                Box::pin(send_telegram(tok, id, txt))
            }).await
        }
        _ => usage(),
    }
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
        // col: 0=name 1=reinvested% 2=precentage 3=amount_reinvested 4=amount_distributed
        //      5=primary 6=pivot 7=usd 8=pivots 9=tweet_url 10=tx_url 11=send 12=flipped
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

    // ---- parse_row ---------------------------------------------------------

    #[test]
    fn test_parse_row_normal() -> ErrStr<()> {
        let row = parse_row(&make_row("α", "14492", "yes", "yes"))?.unwrap();
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
        let row = parse_row(&make_row("τ", "2004", "no", "yes"))?.unwrap();
        assert!(!row.send, "send=no should parse as false");
        Ok(())
    }

    #[test]
    fn test_parse_row_flipped_no() -> ErrStr<()> {
        let row = parse_row(&make_row("γ", "42910", "yes", "no"))?.unwrap();
        assert!(!row.flipped, "flipped=no should parse as false");
        Ok(())
    }

    #[test]
    fn test_parse_row_amount_zero_skipped() -> ErrStr<()> {
        assert!(
            parse_row(&make_row("σ", "0", "yes", "yes"))?.is_none(),
            "amount=0 row should be skipped"
        );
        Ok(())
    }

    #[test]
    fn test_parse_row_column_header_skipped() -> ErrStr<()> {
        let header = "name\treinvested %\tprecentage\tamount reinvested\tamount distributed\t\
                      primary\tpivot\tUSD-value\tnumber of pivots closed\ttweet url\ttx url\tsend?\tflipped";
        assert!(parse_row(header)?.is_none(), "column header row should be skipped");
        Ok(())
    }

    #[test]
    fn test_parse_row_blank_skipped() -> ErrStr<()> {
        assert!(parse_row("")?.is_none(),   "blank line should be skipped");
        assert!(parse_row("  ")?.is_none(), "whitespace line should be skipped");
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
        let err = parse_row(&make_row("ψ", "not-a-number", "yes", "yes"))
            .unwrap_err();
        assert!(err.contains("invalid amount"), "should error loudly, not skip");
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


    create_testing!("quiz11::a_reinvested", "", true);

    run!("mock_process_csv", {
        // col: 0=name 1=reinvested% 2=precentage 3=amount_reinvested 4=amount_distributed
        //      5=primary 6=pivot 7=usd 8=pivots 9=tweet_url 10=tx_url 11=send 12=flipped
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

use std::pin::Pin;
use reqwest::Client;
use book::{
    err_utils::ErrStr,
    utils::{ get_args, get_env },
};


//============================================================================
//----- Version/ App_Name/ Usage ---------------------------------------------
//============================================================================
fn version()  -> &'static str { "2.00" }
fn app_name() -> &'static str { "distributed" }

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
pub struct DistributionRow {
    pub name:    String,
    pub amount:  u64,
    pub primary: String,
    pub pivot:   String,
    pub url:     String,  // tweet url
    pub tx_url:  String,
    pub send:    bool,
}

/// Returns `Ok(None)` for rows that should be skipped:
///   - blank lines
///   - the column-name header row               (col 0 == "name")
///   - data rows where amount distributed == 0  (handled by `reinvested`, not `distributed`)
/// Returns `Err` only for rows that look like data but are malformed.
pub fn parse_row(line: &str) -> ErrStr<Option<DistributionRow>> {
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
    let amount  = cols[4].trim();
    let primary = cols[5].trim();
    let pivot   = cols[6].trim();
    let url     = cols[9].trim();
    let tx_url  = cols[10].trim();
    let send_s  = cols[11].trim().to_lowercase();

    // column-name header row
    if name == "name" {
        return Ok(None);
    }

    // skip investors with nothing distributed this run
    let amount_val: u64 = match amount.parse() {
        Ok(v) => v,
        Err(e) => return Err(format!(
            "row '{name}': invalid amount distributed '{amount}': {e}"
        )),
    };
    if amount_val == 0 {
        return Ok(None);
    }

    let send = send_s == "yes" || send_s == "true";

    Ok(Some(DistributionRow {
        name:    name.to_string(),
        amount:  amount_val,
        primary: primary.to_string(),
        pivot:   pivot.to_string(),
        url:     url.to_string(),
        tx_url:  tx_url.to_string(),
        send,
    }))
}

//============================================================================
//----- Message Building and Sending -----------------------------------------
//============================================================================
pub fn build_message(row: &DistributionRow) -> String {
    format!(
        "I close an {primary}-on-{pivot} pivot (please see the twitter post: {tweet_url}). \
         I sent {amount} {primary} to you; tx_id: {tx_url}",
        primary   = row.primary,
        pivot     = row.pivot,
        tweet_url = row.url,
        amount    = row.amount,
        tx_url    = row.tx_url,
    )
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

        let msg = build_message(&row);
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
    eprintln!("{}, version: {}\n", app_name(), version());
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

    fn make_row(name: &str, amount: &str, send: &str) -> String {
        // col: 0=name 1=reinvested% 2=precentage 3=amount_reinvested 4=amount_distributed
        //      5=primary 6=pivot 7=usd 8=pivots 9=tweet_url 10=tx_url 11=send 12=flipped
        format!(
            "{name}\t0%\t10.25%\t0\t{amount}\tBTC\tUNDEAD\t$35.66\t15\t\
             https://x.com/pivocateur/status/2069591552733712719\t\
             https://snowtrace.io/tx/0x04454ba7f8484359d821f18a5c5e1e6334fa43c416ec345d1de6df10c3e13765\t\
             {send}\tyes"
        )
    }

    fn make_distribution(name: &str, amount: u64, send: bool) -> DistributionRow {
        DistributionRow {
            name:    name.to_string(),
            amount,
            primary: "BTC".to_string(),
            pivot:   "UNDEAD".to_string(),
            url:     "https://x.com/pivocateur".to_string(),
            tx_url:  "https://snowtrace.io/tx/0xabc".to_string(),
            send,
        }
    }

    // ---- parse_row ---------------------------------------------------------

    #[test]
    fn test_parse_row_normal() -> ErrStr<()> {
        let row = parse_row(&make_row("γ", "42910", "yes"))?.unwrap();
        assert_eq!(row.name,    "γ");
        assert_eq!(row.amount,  42910);
        assert_eq!(row.primary, "BTC");
        assert_eq!(row.pivot,   "UNDEAD");
        assert!(row.send);
        Ok(())
    }

    #[test]
    fn test_parse_row_send_no() -> ErrStr<()> {
        let row = parse_row(&make_row("τ", "2004", "no"))?.unwrap();
        assert!(!row.send, "send=no should parse as false");
        Ok(())
    }

    #[test]
    fn test_parse_row_amount_zero_skipped() -> ErrStr<()> {
        assert!(
            parse_row(&make_row("α", "0", "yes"))?.is_none(),
            "amount distributed=0 row should be skipped"
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

    #[test]
    fn test_parse_row_amount_invalid_errors() -> ErrStr<()> {
        let err = parse_row(&make_row("ψ", "not-a-number", "yes"))
            .unwrap_err();
        assert!(err.contains("invalid amount distributed"), "should error loudly, not skip");
        Ok(())
    }

    // ---- build_message -----------------------------------------------------

    #[test]
    fn test_build_message_exact() {
        let mut row = make_distribution("γ", 4349, true);
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
    fn test_build_message_token_positions() {
        let mut row = make_distribution("τ", 2004, true);
        row.primary = "USDC".to_string();
        row.pivot   = "UNDEAD".to_string();
        row.tx_url  = "tx".to_string();
        let msg = build_message(&row);
        assert!(msg.contains("USDC-on-UNDEAD"), "token_a-on-token_b must appear");
        assert!(msg.contains("2004 USDC"),      "amount + token_a must appear");
        assert!(msg.contains("tx_id: tx"),      "tx_url must appear after tx_id:");
    }

    #[test]
    fn test_build_message_different_token_pair() {
        let mut row = make_distribution("δ", 1851, true);
        row.primary = "AVAX".to_string();
        row.pivot   = "BTC".to_string();
        let msg = build_message(&row);
        assert!(msg.contains("AVAX-on-BTC"));
        assert!(msg.contains("1851 AVAX"));
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


    create_testing!("quiz11::b_distributed", "", true);

    run!("mock_process_csv", {
        // col: 0=name 1=reinvested% 2=precentage 3=amount_reinvested 4=amount_distributed
        //      5=primary 6=pivot 7=usd 8=pivots 9=tweet_url 10=tx_url 11=send 12=flipped
        let tsv = "name\treinvested %\tprecentage\tamount reinvested\tamount distributed\t\
                   primary\tpivot\tUSD-value\tnumber of pivots closed\ttweet url\ttx url\tsend?\tflipped\n\
                   γ\t0%\t10.25%\t0\t42910\tBTC\tUNDEAD\t$35.66\t15\t\
                   https://x.com/pivocateur/status/2069591552733712719\t\
                   https://snowtrace.io/tx/0xabc\tyes\tyes\n\
                   α\t100%\t3.46%\t14492\t0\tBTC\tUNDEAD\t$12.04\t15\t\
                   https://x.com/pivocateur/status/2069591552733712719\t\
                   https://snowtrace.io/tx/0xdef\tyes\tyes\n";

        let path_buf = std::env::temp_dir().join("distributed_test.tsv");
        let path = path_buf.to_str().ok_or("temp path is not valid UTF-8")?;
        std::fs::write(path, tsv).map_err(|e| e.to_string())?;

        let _ = now(process_csv(path, false, |tok, id, txt| {
            Box::pin(mock_send_telegram(tok, id, txt))
        }))?;
    });
}

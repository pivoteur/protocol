use reqwest::Client;
use book::{
    err_utils::ErrStr,
    utils::{ get_args, get_env },
};


const DEFAULT_TWEET_URL: &str = "x.com/pivocateur";
const DEFAULT_TX_URL:    &str = "asdf";

fn version()  -> &'static str { "1.02" }
fn app_name() -> &'static str { "distributed" }

fn chat_id_for(investor: &str) -> ErrStr<i64> {
    let raw = get_env("INVESTOR_CHAT_IDS")?;
    let map: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("INVESTOR_CHAT_IDS is not valid JSON: {e}"))?;
    map.get(investor)
        .and_then(|v| v.as_i64())
        .ok_or_else(|| format!("unknown investor/ chat id doesn't exist: {investor}"))
}

fn usage() -> ErrStr<()> {
    eprintln!(
        "Usage: {} <investor> <token_a> <token_b> <amount> <tweet_url> <tx_url> <send>",
        app_name()
    );
    eprintln!("  investor  : investor name / Telegram chat");
    eprintln!("  token_a   : distributed token, left side of pool  (e.g. USDC)");
    eprintln!("  token_b   : paired token,      right side of pool (e.g. UNDEAD)");
    eprintln!("  amount    : amount distributed to investor         (e.g. 0.4349)");
    eprintln!("  tweet_url : tweet URL          (default: {DEFAULT_TWEET_URL})");
    eprintln!("  tx_url    : snowtrace tx URL   (default: {DEFAULT_TX_URL})");
    eprintln!("  send      : let Robbie send message?               (e.g. true/false)");
    Err("Need <investor> <token_a> <token_b> <amount> <tweet_url> <tx_url> <send> arguments".to_string())
}

//----- Message Building -----------------------------------------------------
pub fn build_message(
    token_a:   &str,
    token_b:   &str,
    tweet_url: &str,
    amount:    &str,
    tx_url:    &str,
) -> String {
    format!(
        "I close an {token_a}-on-{token_b} pivot (please see the twitter post: {tweet_url}). \
         I sent {amount} {token_a} to you; tx_id: {tx_url}"
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

pub async fn runoff_with_args() -> ErrStr<()> {
    eprintln!("{}, version: {}\n", app_name(), version());
    let args = get_args();
    match args.as_slice() {
        [investor, token_a, token_b, amount, tweet_url, tx_url, send] => {
            let msg = build_message(token_a, token_b, tweet_url, amount, tx_url);
            let do_send = send.parse::<bool>()
                .map_err(|_| format!("send must be true or false, got: {send}"))?;
            if do_send {
                let bot_token = get_env("REINVESTED_BOT")?;
                let chat_id   = chat_id_for(investor)?;
                send_telegram(&bot_token, chat_id, &msg).await?;
            }
            println!("{msg}");
            Ok(())
        }
        _ => usage(),
    }
}
//----- UNIT TESTS ------------------------------------------------------------------------
#[cfg(test)]
mod unit_tests {
    use super::*;


    #[test]
    fn test_exact_sample_message() {
        let msg = build_message(
            "USDC",
            "UNDEAD",
            "x.com/pivocateur/status/2054570565474635869",
            "0.4349",
            "snowtrace.io/tx/0x04454ba7f8484359d821f18a5c5e1e6334fa43c416ec345d1de6df10c3e13765",
        );
        assert_eq!(
            msg,
            "I close an USDC-on-UNDEAD pivot \
             (please see the twitter post: x.com/pivocateur/status/2054570565474635869). \
             I sent 0.4349 USDC to you; \
             tx_id: snowtrace.io/tx/0x04454ba7f8484359d821f18a5c5e1e6334fa43c416ec345d1de6df10c3e13765"
        );
    }

    #[test]
    fn test_token_positions() {
        let msg = build_message("USDC", "UNDEAD", "tweet", "1.00", "tx");
        assert!(msg.contains("USDC-on-UNDEAD"), "token_a-on-token_b must appear");
        assert!(msg.contains("1.00 USDC"),      "amount token_a must appear");
        assert!(msg.contains("tx_id: tx"),       "tx_url must appear after tx_id:");
    }

    #[test]
    fn test_different_token_pair() {
        let msg = build_message("AVAX", "BTC", "tweet", "3.14", "tx");
        assert!(msg.contains("AVAX-on-BTC"));
        assert!(msg.contains("3.14 AVAX"));
    }

    #[test]
    fn test_default_urls_appear_correctly() {
        let msg = build_message("USDC", "UNDEAD", DEFAULT_TWEET_URL, "0.5", DEFAULT_TX_URL);
        assert!(msg.contains(DEFAULT_TWEET_URL));
        assert!(msg.contains(DEFAULT_TX_URL));
    }
}
//----- FUNCTIONAL TESTS -------------------------------------------------------------------
#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
    use super::*;
    use paste::paste;
    use book::{ create_testing, utils::now };


    create_testing!("quiz11::b_distributed", "", true);

    run!("mock_build_and_send_message", {
        let chat_id = 0i64;
        let msg     = build_message(
            "USDC",
            "UNDEAD",
            "x.com/pivocateur/status/2054570565474635869",
            "0.4349",
            "asdf",
        );
        let _ = now(mock_send_telegram("mock_token", chat_id, &msg))?;
        println!("{msg}");
    });
}
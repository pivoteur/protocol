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
//----- Telegram Configuration -----------------------------------------------
// ===========================================================================
fn chat_id_for(investor: &str) -> ErrStr<i64> {
    let raw = get_env("INVESTOR_CHAT_IDS")?;
    let map: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("INVESTOR_CHAT_IDS is not valid JSON: {e}"))?;
    map.get(investor)
        .and_then(|v| v.as_i64())
        .ok_or_else(|| format!("unknown investor/ chat id doesn't exist: {investor}"))
}

//----- Possible Errors From Trying To Send A Message -----------------------
// * error 403 means:   bot cannot send to that specific chat ID
// * error 400 means:   bot was kicked from chat

//----- Version/ App_Name/ Usage ---------------------------------------------
fn version()  -> &'static str { "1.02" }
fn app_name() -> &'static str { "reinvested" }
 
fn usage() -> ErrStr<()> {
    eprintln!("Usage: {} <investor> <token_a> <token_b> <pivot_count> <amount> <url> <send>", app_name());
    eprintln!("  investor    : name of investor equals telegram chat (e.g. Pivot Internal Bot)");
    eprintln!("  token_a     : reinvested token, left side of pool   (e.g. AVAX)");
    eprintln!("  token_b     : paired token,    right side of pool   (e.g. BTC)");
    eprintln!("  pivot_count : number of pivots closed               (e.g. 2)");
    eprintln!("  amount      : amount reinvested                     (e.g. 0.59)");
    eprintln!("  url         : tweet URL                             (e.g. x.com/pivocateur)");
    eprintln!("  send     : let Robbie send message?              (e.g. true/false, default: true)");
    Err("Need <investor> <token_a> <token_b> <pivot_count> <amount> <url> <send> arguments".to_string())
}

//----- Message Building and Sending -----------------------------------------
pub fn build_message(
    token_a:     &str,
    token_b:     &str,
    pivot_count: &str,
    amount:      &str,
    url:         &str,
) -> ErrStr<String> {
    let n = parse_id(pivot_count)?;
    let pivots = plural(n, "pivot");
    Ok(format!(
        "I just closed {pivots} {token_a}-on-{token_b} and reinvested \
         {amount} ${token_a} into the {token_b}+{token_a} pivot pool for you; tweet: {url}"
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
//----- Mock Telegram (no network call) -------------------------------------
#[cfg(test)]
pub async fn mock_send_telegram(_bot_token: &str, chat_id: i64, text: &str) -> ErrStr<()> {
    println!("[mock telegram] chat_id={chat_id} | text={text}");
    Ok(())
}

//----- fn runoff_with_args ------------------------------------------------
pub async fn runoff_with_args() -> ErrStr<()> {
    eprintln!("{}, version: {}", app_name(), version());
    let args = get_args();
    match args.as_slice() {
        [investor, token_a, token_b, pivot_count, amount, url, send] => {
            let msg = build_message(token_a, token_b, pivot_count, amount, url)?;
            let do_send = send.parse::<bool>()
                .map_err(|_| format!("send must be true or false, got: {send}"))?;
            if do_send {
                let chat_id      = chat_id_for(investor)?;
                let bot_token = get_env("REINVESTED_BOT")?;
                send_telegram(&bot_token, chat_id, &msg).await?;
            }
            println!("{msg}");
            Ok(())
        }
        _ => usage(),
    }
}
// ===========================================================================
//----- UNIT TESTS -----------------------------------------------------------
// ===========================================================================
#[cfg(test)]
mod unit_tests {
    use super::*;
 

    #[test]
    fn test_exact_sample_message() -> ErrStr<()> {
        let msg = build_message(
          "AVAX", "BTC", "2", "0.59",
            "https://x.com/pivocateur/status/2047688113024086275",
        )?;
        assert_eq!(
            msg,
            "I just closed 2 pivots AVAX-on-BTC and reinvested 0.59 $AVAX \
             into the BTC+AVAX pivot pool for you; \
             tweet: https://x.com/pivocateur/status/2047688113024086275"
        );
        Ok(())
    }
 
    #[test]
    fn test_token_positions() -> ErrStr<()> {
        let msg = build_message("ETH", "BTC", "1", "1.5", "https://x.com/pivocateur")?;
        assert!(msg.contains("ETH-on-BTC"), "should show token_a-on-token_b");
        assert!(msg.contains("BTC+ETH"),    "should show token_b+token_a in pool name");
        assert!(msg.contains("$ETH"),       "should show $token_a as the reinvested token");
        Ok(())
    }
 
    #[test]
    fn test_different_token_pair() -> ErrStr<()> {
        let msg = build_message("SOL", "AVAX", "3", "12.5", "https://x.com/pivocateur")?;
        assert!(msg.contains("3 pivots SOL-on-AVAX"));
        assert!(msg.contains("AVAX+SOL pivot pool"));
        assert!(msg.contains("12.5 $SOL"));
        Ok(())
    }
 
    #[test]
    fn test_usage_returns_err() {
        assert!(usage().is_err());
    }

    #[test]
    fn test_singular_pivot_count() -> ErrStr<()> {
        let msg = build_message("AVAX", "BTC", "1", "0.25", "https://x.com/pivocateur")?;
        assert!(msg.contains("I just closed 1 pivot AVAX-on-BTC"),
        "singular count should interpolate cleanly: {msg}");
        Ok(())
    }
     
    #[test]
    fn test_degenerate_empty_inputs() -> ErrStr<()> {
        let msg = build_message("", "", "0", "0", "")?;
        assert!(msg.contains("I just closed 0"),    "pivot_count slot present");
        assert!(msg.contains("-on-"),               "separator present even with empty tokens");
        assert!(msg.contains("pivot pool for you"), "tail of message intact");
        assert!(msg.contains("tweet:"),             "url label present");
        Ok(())
    }
    
}
// ===========================================================================
//----- FUNCTIONAL TESTS -----------------------------------------------------
// ===========================================================================
#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
    use super::*;
    use paste::paste;
    use book::{
            utils::now,
            create_testing
    };


    create_testing!("quiz11::a_reinvested");

    run!("mock_build_and_send_message", {
        let chat_id = 0i64; // dummy chat id for "moak"
        let msg     = build_message(
            "AVAX", "BTC", "2", "0.59",
            "https://x.com/pivocateur",
        )?;
        let _ = now(mock_send_telegram("mock_token", chat_id, &msg))?;
        println!("{msg}");
    });

}

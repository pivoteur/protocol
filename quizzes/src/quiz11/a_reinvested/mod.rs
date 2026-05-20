use reqwest::Client;
use book::{
    err_utils::ErrStr,
    utils::{ 
        get_args, 
        get_env 
    },
};

//============================================================================
//----- Telegram Configuration -----------------------------------------------
// ===========================================================================
fn chat_id_for(name: &str) -> ErrStr<i64> {
    match name {
        "Pivot_Internal_Bot" => Ok(-1003962016174),
        "someone else 0" => Ok(694206909),
        "someone else 1" => Ok(694206908),
        "someone else 2" => Ok(694206907),
        "someone else 3" => Ok(694206906),
        "someone else 4" => Ok(694206905),
        _ => Err(format!("unknown investor: {name}"))
    }
}
//----- Possible Errors From Trying To Send A Message -----------------------

// * error 403 means:   bot cannot send to that specific chat ID
// * error 400 means:   bot was kicked from chat

//----- Version/ App_Name/ Usage ---------------------------------------------
fn version()  -> &'static str { "1.01" }
fn app_name() -> &'static str { "reinvested" }
 
fn usage() -> ErrStr<()> {
    eprintln!("Usage: {} <investor> <token_a> <token_b> <pivot_count> <amount> <url>", app_name());
    eprintln!("  investor    : name of investor equals telegram chat (e.g. Pivot Internal Bot)");
    eprintln!("  token_a     : reinvested token, left side of pool   (e.g. AVAX)");
    eprintln!("  token_b     : paired token,    right side of pool   (e.g. BTC)");
    eprintln!("  pivot_count : number of pivots closed               (e.g. 2)");
    eprintln!("  amount      : amount reinvested                     (e.g. 0.59)");
    eprintln!("  url         : tweet URL                             (e.g. x.com/pivocateur)");
    Err("Need <investor> <token_a> <token_b> <pivot_count> <amount> <url> arguments".to_string())
}
 
// ===========================================================================
//----- Message Building and Sending -----------------------------------------
// ===========================================================================
pub fn build_message(
    investor:    &str,
    token_a:     &str,
    token_b:     &str,
    pivot_count: &str,
    amount:      &str,
    url:         &str,
) -> String {
    format!(
        "Hey {investor}, I just closed {pivot_count} {token_a}-on-{token_b} pivots for you and reinvested \
         {amount} ${token_a} into the {token_b}+{token_a} pivot pool for you; tweet: {url}"
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
//----- fn runoff_with_args ------------------------------------------------
pub async fn runoff_with_args() -> ErrStr<()> {
    eprintln!("{}, version: {}", app_name(), version());
    let args = get_args();
    match args.as_slice() {
        [investor, token_a, token_b, pivot_count, amount, url] => {
            let chat_id     = chat_id_for(investor)?;
            let msg       = build_message(investor, token_a, token_b, pivot_count, amount, url);
            let bot_token = get_env("REINVESTED_BOT")?;
            send_telegram(&bot_token, chat_id, &msg).await?;
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
    fn test_exact_sample_message() {
        let msg = build_message(
          "Pivot_Internal_Bot", "AVAX", "BTC", "2", "0.59",
            "https://x.com/pivocateur/status/2047688113024086275",
        );
        assert_eq!(
            msg,
            "I closed 2 AVAX-on-BTC pivots and reinvested 0.59 $AVAX \
             into the BTC+AVAX pivot pool for you; \
             tweet: https://x.com/pivocateur/status/2047688113024086275"
        );
    }
 
    #[test]
    fn test_token_positions() {
        let msg = build_message("Pivot_Internal_Bot", "ETH", "BTC", "1", "1.5", "https://x.com/test");
        assert!(msg.contains("ETH-on-BTC"), "should show token_a-on-token_b");
        assert!(msg.contains("BTC+ETH"),    "should show token_b+token_a in pool name");
        assert!(msg.contains("$ETH"),       "should show $token_a as the reinvested token");
    }
 
    #[test]
    fn test_different_token_pair() {
        let msg = build_message("Pivot_Internal_Bot", "SOL", "AVAX", "3", "12.5", "https://x.com/test");
        assert!(msg.contains("3 SOL-on-AVAX pivots"));
        assert!(msg.contains("AVAX+SOL pivot pool"));
        assert!(msg.contains("12.5 $SOL"));
    }
 
    #[test]
    fn test_usage_returns_err() {
        assert!(usage().is_err());
    }

    #[test]
    fn test_singular_pivot_count() {
        let msg = build_message("Pivot_Internal_Bot", "AVAX", "BTC", "1", "0.25", "https://x.com/test");
        assert!(msg.contains("I closed 1 AVAX-on-BTC pivots"),
            "singular count should interpolate cleanly: {msg}");
    }

    #[test]
    fn test_degenerate_empty_inputs() {
        let msg = build_message("Pivot_Internal_Bot", "", "", "0", "0", "");
        assert!(msg.contains("I closed 0"),         "pivot_count slot present");
        assert!(msg.contains("-on-"),               "separator present even with empty tokens");
        assert!(msg.contains("pivot pool for you"), "tail of message intact");
        assert!(msg.contains("tweet:"),             "url label present");
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

    run!("build_and_send_message", {
        let bot_token = get_env("REINVESTED_BOT")?;
        let chat_id   = chat_id_for("Pivot_Internal_Bot")?;
        let msg       = build_message(
            "Pivot_Internal_Bot",
            "AVAX", "BTC", "2", "0.59",
            "https://x.com/pivocateur",
        );
        let _ = now(send_telegram(&bot_token, chat_id, &msg))?;
        println!("{msg}");
    });
}

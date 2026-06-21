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
fn version()  -> &'static str { "1.03" }
fn app_name() -> &'static str { "reinvested" }
 
fn usage() -> ErrStr<()> {
    eprintln!("Usage: {} <investor> <token_a> <token_b> <pivot_count> <amount> <url> <send> <flipped>", app_name());
    eprintln!("  investor    : name of investor equals telegram chat (e.g. Pivot Internal Bot)");
    eprintln!("  token_a     : primary asset, left side of pool      (e.g. ETH)");
    eprintln!("  token_b     : pivot asset,   right side of pool     (e.g. UNDEAD)");
    eprintln!("  pivot_count : number of pivots closed               (e.g. 2)");
    eprintln!("  amount      : amount reinvested                     (e.g. 0.59)");
    eprintln!("  url         : tweet URL                             (e.g. x.com/pivocateur)");
    eprintln!("  send     : let Robbie send message?              (e.g. true/false, default: true)");
    eprintln!("  flipped  : when you trade in the opposite direction (e.g. BTC/AVAX instead of AVAX/BTC)");
    Err("Need <investor> <token_a> <token_b> <pivot_count> <amount> <url> <send> <flipped> arguments".to_string())
}
//----- Message Building and Sending -----------------------------------------
pub fn build_message(
    token_a:     &str,
    token_b:     &str,
    pivot_count: &str,
    amount:      &str,
    url:         &str,
    flipped:     bool,
) -> ErrStr<String> {
    let prim = token_a;
    let piv  = token_b;
    let pool = format!("{prim}+{piv}");
    let (reinvested, trade) = if flipped {
        (piv,  format!("{piv}-on-{prim}"))
    } else {
        (prim, format!("{prim}-on-{piv}"))
    };
    let n      = parse_id(pivot_count)?;
    let noun   = format!("{trade} pivot");
    let pivots = if n == 1 {
        noun.clone()
    } else {
        plural(n, &noun)
    };
    Ok(format!(
        "I close {pivots} (see tweet: {url}). \
         I reinvest {amount} {reinvested} into the {pool} pivot pool for you."
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
        [investor, token_a, token_b, pivot_count, amount, url, send, flipped] => {
            let is_flipped = flipped.parse::<bool>()
                .map_err(|_| format!("flipped must be true or false, got: {flipped}"))?;
            let msg = build_message(token_a, token_b, pivot_count, amount, url, is_flipped)?;
            let do_send = send.parse::<bool>()
                .map_err(|_| format!("send must be true or false, got: {send}"))?;
            if do_send {
                let chat_id   = chat_id_for(investor)?;
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
            "UNDEAD", "USDC", "1", "1552",
            "https://x.com/pivocateur/status/2056884438156398786",
            false,
        )?;
        assert_eq!(
            msg,
            "I close UNDEAD-on-USDC pivot (see tweet: \
             https://x.com/pivocateur/status/2056884438156398786). \
             I reinvest 1552 UNDEAD into the UNDEAD+USDC pivot pool for you."
        );
        Ok(())
    }
 
    #[test]
    fn test_token_positions() -> ErrStr<()> {
        let msg = build_message("ETH", "BTC", "1", "1.5", "https://x.com/pivocateur", false)?;
        assert!(msg.contains("ETH-on-BTC"), "should show prim-on-prop");
        assert!(msg.contains("ETH+BTC"),    "should show prim+prop in pool name");
        assert!(msg.contains("1.5 ETH"),    "should show amount prim");
        Ok(())
    }
 
    #[test]
    fn test_different_token_pair() -> ErrStr<()> {
        let msg = build_message("SOL", "AVAX", "3", "12.5", "https://x.com/pivocateur", false)?;
        assert!(msg.contains("3 SOL-on-AVAX pivots"), "plural pivot count");
        assert!(msg.contains("SOL+AVAX pivot pool"),  "pool order prim+prop");
        assert!(msg.contains("12.5 SOL"),             "amount and prim token");
        Ok(())
    }
 
    #[test]
    fn test_singular_pivot_count() -> ErrStr<()> {
        let msg = build_message("AVAX", "BTC", "1", "0.25", "https://x.com/pivocateur", false)?;
        assert!(msg.contains("AVAX-on-BTC pivot "),
            "singular should not append 's': {msg}");
        Ok(())
    }
     
    #[test]
    fn test_degenerate_empty_inputs() -> ErrStr<()> {
        let msg = build_message("", "", "0", "0", "", false)?;
        assert!(msg.contains("I close"),            "opening phrase present");
        assert!(msg.contains("-on-"),               "separator present even with empty tokens");
        assert!(msg.contains("pivot pool for you"), "tail of message intact");
        assert!(msg.contains("see tweet:"),         "url label present");
        Ok(())
    }

    #[test]
    fn test_flipped_pool_order() -> ErrStr<()> {
        // file is eth-undead.tsv, trade is UNDEAD-on-ETH, reinvested token is UNDEAD (piv)
        let msg = build_message("ETH", "UNDEAD", "1", "500", "https://x.com/pivocateur", true)?;
        assert!(msg.contains("UNDEAD-on-ETH"),  "trade direction is piv-on-prim when flipped");
        assert!(msg.contains("ETH+UNDEAD"),     "pool is always prim+piv");
        assert!(msg.contains("500 UNDEAD"),     "reinvested token is piv when flipped");
        Ok(())
    }

    #[test]
    fn test_normal_flow() -> ErrStr<()> {
        let msg = build_message(
            "ETH", "UNDEAD", "1", "0.75",
            "https://x.com/pivocateur/status/2056884438156398786",
            false,
        )?;
        assert_eq!(
            msg,
            "I close ETH-on-UNDEAD pivot (see tweet: \
             https://x.com/pivocateur/status/2056884438156398786). \
             I reinvest 0.75 ETH into the ETH+UNDEAD pivot pool for you."
        );
        Ok(())
    }

    #[test]
    fn test_flipped_flow() -> ErrStr<()> {
        let msg = build_message(
            "ETH", "UNDEAD", "1", "500",
            "https://x.com/pivocateur/status/2056884438156398786",
            true,
        )?;
        assert_eq!(
            msg,
            "I close UNDEAD-on-ETH pivot (see tweet: \
             https://x.com/pivocateur/status/2056884438156398786). \
             I reinvest 500 UNDEAD into the ETH+UNDEAD pivot pool for you."
        );
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
    use book::{  create_testing, utils::now };
 
    create_testing!("quiz11::a_reinvested", "", true);

    run!("mock_build_and_send_message", {
        let chat_id = 0i64; // dummy chat id for "moak"
        let msg     = build_message(
            "UNDEAD", "USDC", "1", "1552",
            "https://x.com/pivocateur",
            false,
        )?;
        let _ = now(mock_send_telegram("mock_token", chat_id, &msg))?;
        println!("{msg}");
    });
}

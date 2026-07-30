pub mod test_helpers;

use std::pin::Pin;
use csv::{ErrorKind, DeserializeErrorKind};
use reqwest::Client;
use book::{err_utils::ErrStr, utils::get_env};

//============================================================================
//----- Telegram Configuration -----------------------------------------------
//============================================================================
pub fn chat_id_for(investor: &str) -> ErrStr<i64> {
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
pub fn is_ragged_row(e: &csv::Error) -> bool {
    let ans = matches!(
        e.kind(),
        ErrorKind::Deserialize { err, .. } if matches!(err.kind(), DeserializeErrorKind::UnexpectedEndOfRow)
    );
    ans
}

pub fn parse_bool_cell(field: &str, raw: &str) -> ErrStr<bool> {
    match raw.trim().to_lowercase().as_str() {
        "yes" | "true"  => Ok(true),
        "no"  | "false" => Ok(false),
        other => Err(format!(
            "column '{field}': unrecognized value '{other}'. Expected yes/no/true/false."
        )),
    }
}

//============================================================================
//----- Message Sending -------------------------------------------------
//============================================================================
pub type SendFuture<'a> = Pin<Box<dyn std::future::Future<Output = ErrStr<()>> + Send + 'a>>;

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

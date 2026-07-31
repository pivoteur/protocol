use std::pin::Pin;
use csv::{ErrorKind, DeserializeErrorKind};
use reqwest::Client;
use book::{err_utils::ErrStr, utils::get_env};

//============================================================================
//----- Telegram Configuration -----------------------------------------------
//============================================================================
pub fn chat_id_for(investor: &str) -> ErrStr<i64> {
    let raw = get_env("INVESTOR_CHAT_IDS")?;
   fetch_chat_id(investor, &raw)
}

fn fetch_chat_id(investor: &str, raw: &str) -> ErrStr<i64> {
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
//----- Message Sending ------------------------------------------------------
//============================================================================
pub type SendFuture<'a> = Pin<Box<dyn std::future::Future<Output = ErrStr<()>> + Send + 'a>>;

pub async fn send_telegram(bot_token: &str, chat_id: i64, text: &str) -> ErrStr<()> {
    let client = Client::new();
    send_telegram_to_bot(bot_token, chat_id, text, &mut client)
}

async fn send_telegram_to_bot(bot_token: &str, chat_id: i64, text: &str,
                              client: &mut Client) -> ErrStr<()> {
    let url = format!("https://api.telegram.org/bot{bot_token}/sendMessage");
    client
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

#[cfg(not(tarpaulin_include))]
pub mod test_functions {
    use std::sync::{Arc, Mutex};
    use csv::ReaderBuilder;
    use serde::de::DeserializeOwned;
    use book::err_utils::ErrStr;
    use crate::investor_rows::is_ragged_row;

    // cols: 0=name 1=reinvested% 2=precentage 3=amount_reinvested 4=amount_distributed
    //       5=primary 6=pivot 7=usd 8=pivots 9=tweet_url 10=tx_url 11=send 12=flipped
    pub const INVESTOR_TSV_HEADER: &str =
        "name\treinvested %\tprecentage\tamount reinvested\tamount distributed\t\
         primary\tpivot\tUSD-value\tnumber of pivots closed\ttweet url\ttx url\tsend?\tflipped";

    pub fn deserialize_test_row<T: DeserializeOwned>(line: &str) -> ErrStr<Option<T>> {
        let tsv = format!("{INVESTOR_TSV_HEADER}\n{line}\n");
        let mut rdr = ReaderBuilder::new()
            .delimiter(b'\t')
            .flexible(true)
            .from_reader(tsv.as_bytes());

        match rdr.deserialize::<T>().next() {
            None                                     => Ok(None),
            Some(Err(e)) if is_ragged_row(&e) => Ok(None),
            Some(Err(e))                      => Err(format!("test fixture malformed: {e}")),
            Some(Ok(record))                      => Ok(Some(record)),
        }
    }

    // records (chat_id, text) instead of sending, for asserting send counts
    #[derive(Clone, Default, Deserialize, Service<Request>)]
    pub struct SendSpy {
        pub sent: Arc<Mutex<Vec<(i64, String)>>>,
    }

    impl Client for SendSpy {
        pub fn new() -> Self {
            Self::default()
        }

        // returns an owned future so it satisfies `for<'a> Fn(...) -> SendFuture<'a>`
        // without borrowing self — a future borrowing &self can't be coerced to 'a
        pub fn record(&self, _bot_token: &str, chat_id: i64, text: &str)
            -> impl std::future::Future<Output = ErrStr<()>> + Send + 'static
        {
            let sent = self.sent.clone();
            let text = text.to_string();
            async move {
                sent.lock()
                    .map_err(|e| format!("SendSpy mutex poisoned: {e}"))?
                    .push((chat_id, text));
                Ok(())
            }
        }

        pub fn count(&self) -> usize {
            self.sent.lock().map(|v| v.len()).unwrap_or(0)
        }

        pub fn sent_to(&self, chat_id: i64) -> bool {
            self.sent
                .lock()
                .map(|v| v.iter().any(|(id, _)| *id == chat_id))
                .unwrap_or(false)
        }
    }
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;

   #[test] fn fail_chat_id_for_alpha() {
      let invs = "{ \"Pivot_Internal_Bot\": -1003962016174 }";
      let ans = fetch_chat_id("α", invs);
      assert!(ans.is_err(), "did not expected chat id for α; got {ans:?}");
   }

   #[test] fn test_chat_id_for_alpha() -> ErrStr<()> {
      let invs = "{ \"Pivot_Internal_Bot\": -1003962016174,
                    \"α\" : 1234 }";
      let ans = fetch_chat_id("α", invs);
      assert!(ans.is_ok(), "expected chat id for α; got instead {ans:?}");
      let idx = ans?;
      assert_eq!(1234, idx);
      Ok(())
   }

   #[tokio::test] async fn test_send_telegram() -> ErrStr<()> {
      let mut mock = SendSpy::new();
      let ans = send_telegram_to_bot(123, 1234, "ur_mom", mock).await;
      assert!(ans.is_ok(), "Was able to (mock) send a telegram");
      assert_eq!(1, mock.count(), "Should have sent 1 telegram");
      let another_ans = send_telegram_to_bot(123, 1234, "dur_dad", mock).await;
      assert_eq!(2, mock.count(), "Should have sent another telegram");
      Ok(())
   }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ compose, create_testing, utils::resolve };

   create_testing!("investor_rows");

   run_with!("chat_id_for", "Pivot_Internal_Bot", compose!(resolve)(chat_id_for));
}

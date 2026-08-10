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
    match e.kind() {
        ErrorKind::Deserialize { err, .. } => matches!(err.kind(), DeserializeErrorKind::UnexpectedEndOfRow),
        _ => false,
    }
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
//----- Message Sending -------------------------------------------------------
//============================================================================
// Shared future type returned by both production send_telegram and test-only SendSpy::record.
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

#[cfg(not(tarpaulin_include))]
pub mod test_functions {
    use std::sync::{Arc, Mutex};
    use csv::ReaderBuilder;
    use serde::de::DeserializeOwned;
    use book::err_utils::ErrStr;
    use super::{is_ragged_row, SendFuture};

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
            None                               => Ok(None),
            Some(Err(e)) if is_ragged_row(&e)  => Ok(None),
            Some(Err(e))                       => Err(format!("test fixture malformed: {e}")),
            Some(Ok(record))                   => Ok(Some(record)),
        }
    }

    /// Records sends instead of making them, so tests can assert on what would have been sent.
    ///
    /// ```ignore
    /// let spy = SendSpy::new();
    /// let spy_for_process = spy.clone();
    /// process_tsv(path, true, move |tok, id, txt| {
    ///     Box::pin(spy_for_process.record(tok, id, txt))
    /// }).await?;
    /// assert!(spy.sent_to(1234));
    /// ```
    #[derive(Clone, Default)]
    pub struct SendSpy {
        pub sent: Arc<Mutex<Vec<(i64, String)>>>,
    }

    impl SendSpy {
        pub fn new() -> Self {
            Self::default()
        }

        // Matches the send_fn signature so it drops into the same closure slot as send_telegram.
        pub fn record(&self, _bot_token: &str, chat_id: i64, text: &str) -> SendFuture<'static> {
            let sent = self.sent.clone();
            let text = text.to_string();
            Box::pin(async move {
                sent.lock()
                    .map_err(|e| format!("SendSpy mutex poisoned: {e}"))?
                    .push((chat_id, text));
                Ok(())
            })
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

        pub fn text_for(&self, chat_id: i64) -> Option<String> {
            self.sent
                .lock()
                .ok()?
                .iter()
                .find(|(id, _)| *id == chat_id)
                .map(|(_, t)| t.clone())
        }
    }
}

// ----- UNIT TESTS ----------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;
    use super::test_functions::{deserialize_test_row, SendSpy};
    use book::utils::now;

    // ---- fetch_chat_id / chat_id_for ---------------------------------------

    #[test] fn fail_chat_id_for_alpha() {
        let invs = "{ \"Pivot_Internal_Bot\": -1003962016174 }";
        let ans = fetch_chat_id("α", invs);
        assert!(ans.is_err(), "did not expect chat id for α; got {ans:?}");
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

    #[test] fn fail_fetch_chat_id_malformed_json() {
        let ans = fetch_chat_id("Pivot_Internal_Bot", "{ not valid json");
        let err = ans.unwrap_err();
        assert!(err.contains("not valid JSON"), "should name the actual problem: {err}");
    }

    // Doesn't remove INVESTOR_CHAT_IDS to test the missing-var path, since parallel tests would race on it.
    #[test] fn test_chat_id_for_matches_fetch_chat_id_for_known_investor() -> ErrStr<()> {
        let raw = get_env("INVESTOR_CHAT_IDS")?;
        let via_fetch = fetch_chat_id("Pivot_Internal_Bot", &raw)?;
        let via_chat_id_for = chat_id_for("Pivot_Internal_Bot")?;
        assert_eq!(via_fetch, via_chat_id_for);
        Ok(())
    }

    // ---- parse_bool_cell ----------------------------------------------------

    #[test] fn test_parse_bool_cell_yes() -> ErrStr<()> {
        assert!(parse_bool_cell("send", "yes")?);
        Ok(())
    }

    #[test] fn test_parse_bool_cell_true() -> ErrStr<()> {
        assert!(parse_bool_cell("send", "true")?);
        Ok(())
    }

    #[test] fn test_parse_bool_cell_no() -> ErrStr<()> {
        assert!(!parse_bool_cell("send", "no")?);
        Ok(())
    }

    #[test] fn test_parse_bool_cell_false() -> ErrStr<()> {
        assert!(!parse_bool_cell("send", "false")?);
        Ok(())
    }

    #[test] fn test_parse_bool_cell_invalid_errors() {
        let err = parse_bool_cell("send", "maybe").unwrap_err();
        assert!(err.contains("send"),               "error should name the field: {err}");
        assert!(err.contains("maybe"),               "error should show the bad value: {err}");
        assert!(err.contains("yes/no/true/false"),   "error should show allowed values: {err}");
    }

    // ---- is_ragged_row (exercised via deserialize_test_row) -----------------
    // Named struct, not a tuple, since std only derives Debug/PartialEq for tuples up to 12 elements.
    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct RowStub {
        name: String,
        #[serde(rename = "number of pivots closed")]
        pivots: String,
        #[serde(rename = "tweet url")]
        tweet_url: String,
        #[serde(rename = "send?")]
        send: String,
        flipped: String,
    }

    #[test] fn test_short_row_is_ragged_and_skipped() -> ErrStr<()> {
        // A 5-column row is too short and should be skipped, not hard-errored.
        let short = "α\t100%\t3.46%\t14492\t0";
        let ans: Option<RowStub> = deserialize_test_row(short)?;
        assert_eq!(ans, None, "a too-short row should be treated as skippable");
        Ok(())
    }

    #[test] fn test_full_row_deserializes() -> ErrStr<()> {
        let full = "α\t100%\t3.46%\t14492\t0\tBTC\tUNDEAD\t$12.04\t15\t\
                     https://x.com/pivocateur/status/1\t\
                     https://snowtrace.io/tx/0xabc\tyes\tyes";
        let ans: Option<RowStub> = deserialize_test_row(full)?;
        assert!(ans.is_some(), "a full 13-column row should deserialize");
        Ok(())
    }

    #[test] fn test_genuine_type_error_is_not_treated_as_ragged() {
        // Right column count, but "amount reinvested" isn't a number, so it must error, not be skipped.
        let bad = "α\t100%\t3.46%\tnot-a-number\t0\tBTC\tUNDEAD\t$12.04\t15\t\
                    https://x.com/pivocateur/status/1\t\
                    https://snowtrace.io/tx/0xabc\tyes\tyes";
        let ans: ErrStr<Option<RowStub>> = deserialize_test_row(bad);
        assert!(ans.is_err(), "a genuine type mismatch must propagate, not be skipped");
    }

    // ---- SendSpy --------------------------------------------------------------

    #[test] fn test_send_spy_records_a_call() -> ErrStr<()> {
        let spy = SendSpy::new();
        now(spy.record("tok", 1234, "hello"))?;
        assert_eq!(spy.count(), 1);
        assert!(spy.sent_to(1234));
        assert_eq!(spy.text_for(1234), Some("hello".to_string()));
        Ok(())
    }

    #[test] fn test_send_spy_records_multiple_calls() -> ErrStr<()> {
        let spy = SendSpy::new();
        now(spy.record("tok", 1234, "first"))?;
        now(spy.record("tok", 5678, "second"))?;
        assert_eq!(spy.count(), 2);
        assert!(spy.sent_to(1234));
        assert!(spy.sent_to(5678));
        assert!(!spy.sent_to(9999), "an unrecorded chat_id must not read as sent-to");
        Ok(())
    }
}

// ----- FUNCTIONAL TESTS -----------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
    use super::*;
    use paste::paste;
    use book::{ compose, create_testing, utils::resolve };

    create_testing!("investor_rows");

    // ---- chat_id_for --------------------------------------------------------
    // Calls the real function against the real INVESTOR_CHAT_IDS env var and panics loudly on error.
    run_with!("chat_id_for", "Pivot_Internal_Bot", compose!(resolve)(chat_id_for));

    // ---- send_telegram (smoke test) -----------------------------------------
    // Fires a real message via the real bot token so cargo test does not spam the chat by default.
    // Run it with: cargo test smoke_test_send_telegram_real -- --ignored
    #[ignore]
    #[tokio::test]
    async fn smoke_test_send_telegram_real() -> ErrStr<()> {
        let bot_token = get_env("REINVESTED_BOT")?;
        let chat_id   = chat_id_for("Pivot_Internal_Bot")?;
        send_telegram(&bot_token, chat_id, "[smoke test] investor_rows::send_telegram is alive").await?;
        println!("smoke_test_send_telegram_real:...ok — check the Doug+Paris chat");
        Ok(())
    }
}

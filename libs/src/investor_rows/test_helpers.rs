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
            None                              => Ok(None),
            Some(Err(e)) if is_ragged_row(&e) => Ok(None),
            Some(Err(e))                      => Err(format!("test fixture malformed: {e}")),
            Some(Ok(record))                  => Ok(Some(record)),
        }
    }

    // records (chat_id, text) instead of sending, for asserting send counts
    #[derive(Clone, Default)]
    pub struct SendSpy {
        pub sent: Arc<Mutex<Vec<(i64, String)>>>,
    }

    impl SendSpy {
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

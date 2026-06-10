use chrono::NaiveDate;
use serde::{Deserialize, Deserializer};
use serde_with::{serde_as, DisplayFromStr};
use std::error::Error;

// -----------------------------------------------------------------------------
// CUSTOM SERDE PARSERS FOR CLEANING FIELDS
// -----------------------------------------------------------------------------

// Strips characters like '$', '%', and ',' before parsing into a float
fn deserialize_clean_float<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let cleaned = s.replace('$', "").replace('%', "").replace(',', "");
    cleaned.trim().parse::<f64>().map_err(serde::de::Error::custom)
}

// Splits semi-colon separated lists (e.g. "27;29") into a Vec<i32>
fn deserialize_semicolon_list<'de, D>(deserializer: D) -> Result<Vec<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    if s.trim().is_empty() {
        return Ok(Vec::new());
    }
    s.split(';')
        .map(|val| val.trim().parse::<i32>().map_err(serde::de::Error::custom))
        .collect()
}

// -----------------------------------------------------------------------------
// 📇 THE STRUCT (Data Layout)
// -----------------------------------------------------------------------------
#[derive(Debug, Deserialize)]
#[serde_as]
pub struct CryptoCallRecord {
    pub ix: i32,
    pub pool: String,
    pub open_pivots: i32,
    #[serde_as(as = "DisplayFromStr")]
    pub last_pivot_on_dt: NaiveDate,
    #[serde_as(as = "DisplayFromStr")]
    pub opened: NaiveDate,
    #[serde(deserialize_with = "deserialize_semicolon_list")]
    pub ids: Vec<i32>,
    pub close_id: i32,
    #[serde_as(as = "DisplayFromStr")]
    pub close_date: NaiveDate,
    #[serde(rename = "from")] // Re-maps the reserved Rust keyword safely
    pub from_token: String,
    pub from_blockchain: String,
    #[serde(deserialize_with = "deserialize_clean_float")]
    pub amount1: f64,
    #[serde(deserialize_with = "deserialize_clean_float")]
    pub virtual_amount: f64, // Renamed 'virtual' as it is a reserved word in some contexts
    #[serde(deserialize_with = "deserialize_clean_float")]
    pub quote1: f64,
    #[serde(deserialize_with = "deserialize_clean_float")]
    pub val1: f64,
    #[serde(deserialize_with = "deserialize_clean_float")]
    pub gain_10_percent: f64,
    pub pivot_token: String,
    pub pivot_blockchain: String,
    #[serde(deserialize_with = "deserialize_clean_float")]
    pub pivot_close_price: f64,
    #[serde(deserialize_with = "deserialize_clean_float")]
    pub pivot_amount: f64,
    pub proposed_token: String,
    pub proposed_blockchain: String,
    #[serde(deserialize_with = "deserialize_clean_float")]
    pub proposed_close_price: f64,
    #[serde(deserialize_with = "deserialize_clean_float")]
    pub proposed_amount: f64,
    #[serde(deserialize_with = "deserialize_clean_float")]
    pub roi: f64,
    #[serde(deserialize_with = "deserialize_clean_float")]
    pub apr: f64,
}

// -----------------------------------------------------------------------------
// 📑 THE PARSER
// -----------------------------------------------------------------------------
pub fn parse_crypto_csv(csv_data: &str) -> Result<Vec<CryptoCallRecord>, Box<dyn Error>> {
    let mut reader = csv::Reader::from_reader(csv_data.as_bytes());
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: CryptoCallRecord = result?;
        records.push(record);
    }

    Ok(records)
}

// -----------------------------------------------------------------------------
// 🚀 RUNTIME VERIFICATION
// -----------------------------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;

   use paste::paste;
   use book::{create_testing,err_utils::ErrStr};

   create_testing!("quizzes::quiz08::e_offrian");

   run!("parse_calls", {
      let raw_csv_data = "\
ix,pool,open_pivots,last_pivot_on_dt,opened,ids,close_id,close_date,from,from_blockchain,amount1,virtual,quote1,val1,gain_10_percent,pivot_token,pivot_blockchain,pivot_close_price,pivot_amount,proposed_token,proposed_blockchain,proposed_close_price,proposed_amount,roi,apr
1,BTC+USDC,10,2026-04-16,2026-04-15,27;29,8,2026-06-10,BTC,Avalanche,0,0.452206,$81812.00,$36995.88,0.4974266,USDC,Avalanche,$1.00,37005.758,BTC,Avalanche,$61419.00,0.6023795,33.21%,216.45%
2,BTC+UNDEAD,20,2026-04-09,2026-02-07,3;5;8;10;28;32;34;36;40,15,2026-06-10,UNDEAD,Avalanche,2189400,540280.56,$0.001782,$4863.69,3002648.5,BTC,Avalanche,$61419.00,0.0646658,UNDEAD,Avalanche,$0.000960,4135559.8,51.50%,152.84%";

      let parsed_records = parse_crypto_csv(raw_csv_data)?;

      for record in parsed_records {
         println!(
            "Pool: {:<11} | ROI: {:>5}% | IDs Vector: {:?}",
            record.pool, record.roi, record.ids
         );
      }
   });
}


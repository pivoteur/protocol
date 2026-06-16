use chrono::NaiveDate;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde_with::{serde_as, DisplayFromStr};

use book::{
   currency::usd::USD,
   err_utils::{ErrStr,err_or},
   num::percentage::Percentage
};

use super::{
   blockchains::Blockchain,
   pools::Pool
};

#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct Call {
    pub ix: usize,
    #[serde_as(as = "DisplayFromStr")]
    pub pool: Pool,
    pub open_pivots: usize,
    #[serde_as(as = "DisplayFromStr")]
    pub last_pivot_on_dt: NaiveDate,
    #[serde_as(as = "DisplayFromStr")]
    pub opened: NaiveDate,
    #[serde(deserialize_with = "deserialize_semicolon_list")]
    #[serde(serialize_with = "serialize_semicolon_list")]
    pub ids: Vec<usize>,
    pub close_id: usize,
    #[serde_as(as = "DisplayFromStr")]
    pub close_date: NaiveDate,
    #[serde(rename = "from")] // Re-maps the reserved Rust keyword safely
    pub from_token: String,
    #[serde_as(as = "DisplayFromStr")]
    pub from_blockchain: Blockchain,
    pub amount1: f32,
    #[serde(rename = "virtual")]
    pub virtual_amount: f32,
    #[serde_as(as = "DisplayFromStr")]
    pub quote1: USD,
    #[serde_as(as = "DisplayFromStr")]
    pub val1: USD,
    pub gain_10_percent: f32,
    pub pivot_token: String,
    #[serde_as(as = "DisplayFromStr")]
    pub pivot_blockchain: Blockchain,
    #[serde_as(as = "DisplayFromStr")]
    pub pivot_close_price: USD,
    pub pivot_amount: f32,
    pub proposed_token: String,
    #[serde_as(as = "DisplayFromStr")]
    pub proposed_blockchain: Blockchain,
    #[serde_as(as = "DisplayFromStr")]
    pub proposed_close_price: USD,
    pub proposed_amount: f32,
    #[serde_as(as = "DisplayFromStr")]
    pub roi: Percentage,
    #[serde_as(as = "DisplayFromStr")]
    pub apr: Percentage
}

fn deserialize_semicolon_list<'de, D>(deserializer: D)
      -> Result<Vec<usize>, D::Error> where D: Deserializer<'de> {
    let s: String = Deserialize::deserialize(deserializer)?;
    if s.trim().is_empty() { return Ok(Vec::new()); }
    s.split(';')
     .map(|val| val.trim().parse::<usize>().map_err(serde::de::Error::custom))
     .collect()
}

fn serialize_semicolon_list<S>(data: &Vec<usize>, serializer: S)
      -> Result<S::Ok, S::Error> where S: Serializer {

   // 1. Convert each usize to a String
   let parts: Vec<String> = data.iter().map(|x| x.to_string()).collect();

   // 2. Join the elements using a semicolon
   let joined = parts.join(";");
        
   // 3. Serialize as a single string primitive
   serializer.serialize_str(&joined)
}

pub fn parse_calls(csv_data: &str) -> ErrStr<Vec<Call>> {
   let mut reader = csv::Reader::from_reader(csv_data.as_bytes());
   let mut records = Vec::new();
   let mut ix = 0;
   for result in reader.deserialize() {
      ix += 1;
      let record: Call = err_or(result, &format!("Cannot parse row {ix}"))?;
      records.push(record);
   }

   Ok(records)
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use book::string_utils::s;

   fn sample_calls() -> String {
      s("ix,pool,open_pivots,last_pivot_on_dt,opened,ids,close_id,close_date,from,from_blockchain,amount1,virtual,quote1,val1,gain_10_percent,pivot_token,pivot_blockchain,pivot_close_price,pivot_amount,proposed_token,proposed_blockchain,proposed_close_price,proposed_amount,roi,apr
1,BTC+USDC,10,2026-04-16,2026-04-15,27;29,8,2026-06-10,BTC,Avalanche,0,0.452206,$81812.00,$36995.88,0.4974266,USDC,Avalanche,$1.00,37005.758,BTC,Avalanche,$61419.00,0.6023795,33.21%,216.45%
2,BTC+UNDEAD,20,2026-04-09,2026-02-07,3;5;8;10;28;32;34;36;40,15,2026-06-10,UNDEAD,Avalanche,2189400,540280.56,$0.001782,$4863.69,3002648.5,BTC,Avalanche,$61419.00,0.0646658,UNDEAD,Avalanche,$0.000960,4135559.8,51.50%,152.84%")
   }

   #[test] fn test_parse_calls_ok() {
      let calls = parse_calls(&sample_calls());
      assert!(calls.is_ok());
   }
}

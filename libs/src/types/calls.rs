use chrono::NaiveDate;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde_with::{serde_as, DisplayFromStr};

use book::{
   currency::usd::USD,
   err_utils::{ErrStr,err_or},
   num::percentage::Percentage
};

use super::{ blockchains::Blockchain, pools::Pool, util::Id };

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Call {
    pub ix: Id,
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
   use crate::fetchers::test_helpers::test_functions::fetch_local_data;

   #[test] fn test_parse_calls_ok() -> ErrStr<()> {
      let sample_calls = fetch_local_data("../quizzes", "sample_calls.csv")?;
      let calls = parse_calls(&sample_calls);
      assert!(calls.is_ok());
      Ok(())
   }
}

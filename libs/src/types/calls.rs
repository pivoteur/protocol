use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use serde_with::{serde_as, DisplayFromStr};

use book::{
   currency::usd::USD,
   err_utils::{ErrStr,err_or},
   num::percentage::Percentage
};

use super::{
   blockchains::Blockchain,
   pools::Pool,
   util::Id,
   pivots::opens::Pivot
};
use crate::processors::utils::{
   deserialize_semicolon_list,
   serialize_semicolon_list
};

pub type CallData = (Call, Vec<Pivot>);

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
mod functional_tests {
   use super::*;
   use crate::fetchers::test_helpers::test_functions::fetch_local_data;
   use paste::paste;
   use book::{ create_testing, csv_utils::as_csv };

   create_testing!("types::calls");

   run!("parse_calls", {
      let sample_calls = fetch_local_data("../quizzes", "sample_calls.csv")?;
      let calls = parse_calls(&sample_calls)?;
      println!("Calls from csv-data:\n{}", as_csv(&calls, true)?);
   });
}

#[cfg(not(tarpaulin_include))] 
pub mod test_data {
   use super::*;
   use book::currency::usd::mk_usd;
   use crate::fetchers::test_helpers::test_functions::{
      parse_test_pivots_from_file,
      fetch_local_data
   }; 
      
   pub fn target() -> USD { mk_usd(1000.0) }
   pub fn tenk() -> USD { mk_usd(10000.0) }
         
   pub fn sample_call(ix: usize) -> ErrStr<Call> {
      let raw_csv_data = fetch_local_data("../quizzes", "sample_calls.csv")?;
      let calls = parse_calls(&raw_csv_data)?;
      Ok(calls[ix-1].clone()) // ix - 1 because 1 is 0 sometimes. *sigh*
   }     

   fn sample_offrian(relative: &str, ix: Id, pool: &str, file: &str)
         -> ErrStr<CallData> {
      let call = sample_call(ix)?;
      let filename = format!("{relative}/data/{file}.tsv");
      let (opens, _closes) = parse_test_pivots_from_file(pool, &filename)?;
      Ok((call, opens))
   }

   pub fn sample_undead_usdc_offrian(relative: &str) -> ErrStr<CallData> {
      sample_offrian(relative, 1, "undead-usdc", 
                     "sample_undead_usdc_open_pivots")
   }

   pub fn sample_btc_undead_offrian(relative: &str) -> ErrStr<CallData> {
      sample_offrian(relative, 4, "btc-undead", 
                     "sample_btc_undead_open_pivots")
   }
}


use serde::{ Deserialize, Deserializer };

use book::{
   currency::usd::{ USD,mk_usd },
   csv_utils::{CsvWriter,CsvHeader },
   string_utils::s,
   types::filters::Filter
};
use crate::types::measurable::Measurable;

#[derive(Deserialize, Debug)]
pub struct Tokens { pub result: Vec<TokenBalance> }

#[derive(Deserialize, Debug)]
pub struct TokenBalance {
    symbol: String,
    balance: String,
    decimals: Option<u8>,
    token_address: String,

    #[serde(deserialize_with = "parse_float_to_usd")]
    usd_price: USD
}

impl Filter<String> for TokenBalance {
   fn filter(&self) -> String { self.token_address.clone() }
}

impl Measurable for TokenBalance {
   fn sz(&self) -> f32 {
      let bal = self.bal();
      bal.parse().expect(&format!("{} is not a number", bal))
   }
   fn aug(&self) -> f32 { self.usd_price.amount }
}

impl CsvHeader for TokenBalance {
   fn header(&self) -> String { s("symbol,balance,quote,nav") }
}
impl CsvWriter for TokenBalance {
   fn ncols(&self) -> usize { 4 }
   fn as_csv(&self) -> String {
      format!("{},{},{},{}",self.symbol,self.bal(),self.usd_price,self.nav())
   }
}

impl TokenBalance {
   // Formats a raw string balance using the provided decimals
   pub fn bal(&self) -> String {
      let raw_balance = &self.balance;
      let deci = &self.decimals;
      match raw_balance.parse::<f64>() {
         Ok(val) => {
            let dec = deci.unwrap_or(18);
            let formatted = val / 10.0_f64.powi(dec as i32);
            format!("{:.4}", formatted)
            // Truncate to 4 decimal places for readability
         }
         Err(_) => s(raw_balance)
      }
   }

   pub fn nav(&self) -> USD { mk_usd(self.sz() * self.aug()) }
}

fn parse_float_to_usd<'de, D>(deserializer: D) -> Result<USD, D::Error>
      where D: Deserializer<'de> {
    // First, deserialize the JSON field into a standard Rust String
    let s = String::deserialize(deserializer)?;
    
    // Second, parse the string using your own logic
    let parsed_float = s.parse::<f32>().map_err(serde::de::Error::custom)?;
    
    // Finally, return your constructed type
    Ok(mk_usd(parsed_float))
}


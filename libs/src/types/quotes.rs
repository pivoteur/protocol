use std::collections::HashMap;

use chrono::NaiveDate;
use serde::Serialize;
use serde_with::{ serde_as, DisplayFromStr };

use book::{
   currency::usd::mk_usd,
   csv_utils::{CsvHeader,CsvWriter},
   err_utils::ErrStr,
   string_utils::s,
   table_utils::{Table,from_map}
};

use super::{
   aliases::{Aliases,aliases},
   util::Token
};

/// One populates the quotes with the fetcher.

#[serde_as]
#[derive(Clone, Debug, Serialize)]
pub struct Quotes {
   #[serde_as(as = "DisplayFromStr")]
   pub date: NaiveDate,
   quotes: HashMap<Token, f32>,
   #[serde(skip_serializing)]
   pub aliases: Aliases
}

impl CsvWriter for Quotes {
   fn ncols(&self) -> usize { 1 + self.quotes.len() }
   fn as_csv(&self) -> String {
      let vals: Vec<String> =
         self.quotes.values()
                    .map(|quote| format!("{}", mk_usd(*quote)))
                    .collect();
      format!("{},{}", self.date, vals.join(","))
   }
}

impl CsvHeader for Quotes {
   fn header(&self) -> String {
      let toks: Vec<String> =
         self.quotes.keys().map(String::to_string).collect();
      format!("date,{}", toks.join(","))
   }
}

pub fn mk_quotes(dt: &NaiveDate, qts: &[(&str, f32)]) -> Quotes {
   let a = aliases();
   let quotes: HashMap<Token, f32> =
      qts.into_iter().map(|(k,v)| (a.alias(k), *v)).collect();
   Quotes { date: dt.clone(), quotes, aliases: a }
}

impl Quotes {
   pub fn lookup(&self, key: &str) -> ErrStr<f32> {
      let k = s(key);
      self.quotes.get(&self.aliases.alias(&k))
              .ok_or(format!("Unable to find quote for {k}"))
              .copied()
   }
   pub fn as_table(&self) -> Table<usize,Token,f32> {
      from_map(&1, &self.quotes)
   }
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod sample_data {

   use super::{mk_quotes,Quotes};

   use std::iter::once;

   use book::date_utils::yesterday;

   pub fn sample_quotes_maker(q: &[(&str, f32)]) -> Quotes {
      let quotes: Vec<(&str, f32)> =
         once(("USDC", 1.0)).chain(q.into_iter().cloned()).collect();
      mk_quotes(&yesterday(), &quotes)
   }

   pub fn sample_btc_eth_quotes() -> Quotes {
      sample_quotes_maker(&[("BTC", 76603.0), ("ETH", 2086.77)])
   }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {

   use super::*;
   use super::sample_data::sample_btc_eth_quotes;

   use paste::paste;
   use serde_json;

   use book::{ create_testing, err_utils::err_or };

   create_testing!("types::quotes");

   run!("as_table", {
      let qts = sample_btc_eth_quotes();
      println!("quotes are:
{}", qts.as_table().as_csv());
   });

   run!("serialized_quotes", {
      let quotes = sample_btc_eth_quotes();
      let json = err_or(serde_json::to_string_pretty(&quotes),
                        "Could not serialize quotes as JSON")?;
      println!("Converting {quotes:?} to JSON:\n\n{json}");
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use super::sample_data::sample_quotes_maker;

   fn looking(token: &str) -> ErrStr<f32> {
      sample_quotes_maker(&[("BTC", 68732.0)]).lookup(token)
   }

   #[test]
   fn test_lookup_ok() {
      let usdc = looking("USDC");
      assert!(usdc.is_ok());
   }

   #[test]
   fn test_lookup_alias_ok() -> ErrStr<()> {
      let mb_iusd = looking("iUSD");
      assert!(mb_iusd.is_ok());
      let iusd = mb_iusd?;
      assert_eq!(1.0, iusd);
      Ok(())
   }

   #[test]
   fn fail_lookup() {
      let mb_ick = looking("Ick");
      assert!(mb_ick.is_err());
   }
}


use std::collections::HashMap;

use chrono::NaiveDate;

use book::{
   currency::usd::mk_usd,
   csv_utils::{CsvHeader,CsvWriter},
   err_utils::ErrStr,
   table_utils::{Table,from_map}
};

use super::{
   aliases::{Aliases,aliases},
   util::Token
};

/// One populates the quotes with the fetcher.

#[derive(Clone, Debug)]
pub struct Quotes {
   pub date: NaiveDate,
   quotes: HashMap<Token, f32>,
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

pub fn mk_quotes(date: NaiveDate, quotes: HashMap<Token, f32>) -> Quotes {
   Quotes { date, quotes, aliases: aliases() }
}

impl Quotes {
   pub fn lookup(&self, key: &Token) -> ErrStr<f32> {
      self.quotes.get(&self.aliases.alias(key))
              .ok_or(format!("Unable to find quote for {key}"))
              .copied()
   }
   pub fn as_table(&self) -> Table<usize,Token,f32> {
      from_map(&1, &self.quotes)
   }
}

// ----- TESTS -------------------------------------------------------

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use std::iter::once;
   use book::date_utils::yesterday;

   pub fn test_mk_quotes(q: &[(&str, f32)]) -> Quotes {
      let quotes: Vec<(String, f32)> =
         once(&("USDC", 1.0)).chain(q.into_iter())
                             .map(|(a,b)| (a.to_string(), *b))
                             .collect();
      mk_quotes(yesterday(), quotes.into_iter().collect())
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use super::functional_tests::test_mk_quotes;

   fn looking(token: &str) -> ErrStr<f32> {
      test_mk_quotes(&[("BTC", 68732.0)]).lookup(&token.to_string())
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


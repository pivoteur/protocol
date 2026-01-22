use std::collections::HashMap;

use chrono::NaiveDate;

use book::{
   currency::usd::mk_usd,
   csv_utils::CsvWriter,
   err_utils::ErrStr,
};

use super::{
   aliases::{Aliases,aliases},
   util::{Token,CsvHeader}
};

/// One populates the quotes with the fetcher.

#[derive(Clone, Debug)]
pub struct Quotes {
   pub date: NaiveDate,
   quotes: HashMap<Token, f32>,
   aliases: Aliases
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
}


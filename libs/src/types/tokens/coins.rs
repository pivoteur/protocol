use std::ops::AddAssign;

use chrono::NaiveDate;

use book::{
   csv_utils::{CsvHeader,CsvWriter},
   currency::usd::{USD,mk_usd},
   err_utils::ErrStr
};

use crate::types::{
   measurable::{Measurable,tvl},
   quotes::Quotes,
   util::{Blockchain,Token}
};

// ----- ASSETS ----------------------------------------------------------

/// A Coin (an element of Assets) is a Token distinguished by Blockchain
#[derive(Default, Debug, Clone)]
pub struct Coin {
   blockchain: Blockchain,
   token: Token,
   amount: f32,
   quote: USD,
   pub date: NaiveDate
}

impl AddAssign<f32> for Coin {
   fn add_assign(&mut self, rhs: f32) {
      self.amount += rhs;
   }
}

impl Measurable for Coin {
   fn sz(&self) -> f32 { self.amount }
   fn aug(&self) -> f32 { self.quote.amount() }
}

impl CsvHeader for Coin {
   fn header(&self) -> String { format!("date,blockchain,{}", base_header()) }
}
impl CsvWriter for Coin {
   fn ncols(&self) -> usize { 2 + base_sz() }
   fn as_csv(&self) -> String {
      format!("{},{},{}", self.date, self.blockchain, base_csv_values(&self))
   }
}

impl Coin {
   pub fn key(&self) -> (Blockchain, Token) {
      (self.blockchain.clone(), self.token.clone())
   }
   pub fn update_price(&mut self, qts: &Quotes) -> ErrStr<()> {
      let quote = qts.lookup(&self.token)?;
      self.date = qts.date.clone();
      self.quote = mk_usd(quote);
      Ok(())
   }
}

pub fn mk_coin(k: &(Blockchain, Token), amount: f32,
                quote: &USD, date: &NaiveDate) -> Coin {
   let (b, t) = k;
   Coin { blockchain: b.clone(),
           token: t.clone(), 
           amount, 
           quote: quote.clone(),
           date: date.clone() }
}

// ----- PIVOT ASSET -------------------------------------------------------

/// Representation of a coin without the redundant date and blockchain data
#[derive(Debug,Clone)]
pub struct PivotCoin { asset: Coin }

pub fn mk_pivot_coin(asset: Coin) -> PivotCoin { PivotCoin { asset } }

impl CsvHeader for PivotCoin {
   fn header(&self) -> String { base_header() }
}
impl CsvWriter for PivotCoin {
   fn ncols(&self) -> usize { base_sz() }
   fn as_csv(&self) -> String { base_csv_values(&self.asset) }
}

fn base_header() -> String { "token,quote,amount,total".to_string() }
fn base_csv_values(a: &Coin) -> String {
   format!("{},{},{},{}", a.token, a.quote, a.amount, tvl(a))
}
fn base_sz() -> usize { 4 }

impl Measurable for PivotCoin {
   fn sz(&self) -> f32 { self.asset.sz() }
   fn aug(&self) -> f32 { self.asset.aug() }
}

impl PivotCoin {
   pub fn key(&self) -> Token { self.asset.token.clone() }
   pub fn coin(&self) -> Coin { self.asset.clone() }
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod test_data {
   use super::*;
   use book::{ date_utils::yesterday, string_utils::s, utils::now };
   use crate::fetchers::quotes::fetch_quotes;

   pub fn coin(tok: &str, amt: f32) -> ErrStr<Coin> {
      let t = s(tok);
      let yday = yesterday();
      let quotes = now(fetch_quotes(&yday))?;
      let qt = quotes.lookup(&t)?;
      Ok(mk_coin(&(s("Avalanche"), t), amt, &mk_usd(qt), &yday))
   }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use super::test_data::coin;
   use paste::paste;
   use book::{ create_testing, utils::{deref,composer} };

   create_testing!("types::coins");

   run_with!("mk_coin", &coin("BTC", 0.1)?, CsvWriter::as_csv);
   run_with!("mk_pivot_coin", coin("ETH", 3.4)?,
             composer(deref(CsvWriter::as_csv), mk_pivot_coin));
}


use chrono::NaiveDate;

use book::{
   csv_utils::{CsvHeader,CsvWriter},
   currency::usd::USD
};

use crate::types::{
   measurable::{Measurable,tvl},
   util::{Blockchain,Token}
};

// ----- ASSETS ----------------------------------------------------------

/// A Coin (an element of Assets) is a Token distinguished by Blockchain
#[derive(Debug, Clone)]
pub struct Coin {
   blockchain: Blockchain,
   token: Token,
   pub amount: f32,
   pub quote: USD,
   pub date: NaiveDate
}

impl Measurable for Coin {
   fn sz(&self) -> f32 { self.amount }
   fn aug(&self) -> f32 { self.quote.amount }
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
}


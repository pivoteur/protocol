use chrono::NaiveDate;

use book::{
   csv_utils::CsvWriter,
   currency::usd::USD
};

use crate::types::{
   measurable::{Measurable,tvl},
   util::{Blockchain,CsvHeader,Token}
};

// ----- ASSETS ----------------------------------------------------------

/// An Asset (an element of Assets) is a Token distinguished by Blockchain
#[derive(Debug, Clone)]
pub struct Asset {
   blockchain: Blockchain,
   token: Token,
   pub amount: f32,
   quote: USD,
   pub date: NaiveDate
}

impl Measurable for Asset {
   fn sz(&self) -> f32 { self.amount }
   fn aug(&self) -> f32 { self.quote.amount }
}

impl CsvHeader for Asset {
   fn header(&self) -> String { format!("date,blockchain,{}", base_header()) }
}
impl CsvWriter for Asset {
   fn ncols(&self) -> usize { 2 + base_sz() }
   fn as_csv(&self) -> String {
      format!("{},{},{}", self.date, self.blockchain, base_csv_values(&self))
   }
}

impl Asset {
   pub fn key(&self) -> (Blockchain, Token) {
      (self.blockchain.clone(), self.token.clone())
   }
}

pub fn mk_asset(k: &(Blockchain, Token), amount: f32,
                quote: &USD, date: &NaiveDate) -> Asset {
   let (b, t) = k;
   Asset { blockchain: b.clone(),
           token: t.clone(), 
           amount, 
           quote: quote.clone(),
           date: date.clone() }
}

/// Representation of an asset without the redundant date and blockchain data
#[derive(Debug,Clone)]
pub struct PivotAsset { asset: Asset }

pub fn mk_pivot_asset(asset: Asset) -> PivotAsset { PivotAsset { asset } }

impl CsvHeader for PivotAsset {
   fn header(&self) -> String { base_header() }
}
impl CsvWriter for PivotAsset {
   fn ncols(&self) -> usize { base_sz() }
   fn as_csv(&self) -> String { base_csv_values(&self.asset) }
}

fn base_header() -> String { "token,quote,amount,total".to_string() }
fn base_csv_values(a: &Asset) -> String {
   format!("{},{},{},{}", a.token, a.quote, a.amount, tvl(a))
}
fn base_sz() -> usize { 4 }

impl Measurable for PivotAsset {
   fn sz(&self) -> f32 { self.asset.sz() }
   fn aug(&self) -> f32 { self.asset.aug() }
}

impl PivotAsset {
   pub fn key(&self) -> Token { self.asset.token.clone() }
}


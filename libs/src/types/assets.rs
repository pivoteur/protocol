use chrono::NaiveDate;

use book::{
   csv_utils::CsvWriter,
   currency::usd::{USD,mk_usd}
};

use crate::types::{
   measurable::Measurable,
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
   date: NaiveDate
}

impl Measurable for Asset {
   fn sz(&self) -> f32 { self.amount }
   fn aug(&self) -> f32 { self.quote.amount }
}

impl CsvHeader for Asset {
   fn header(&self) -> String {
      "date,blockchain,token,quote,amount,total".to_string()
   }
}
impl CsvWriter for Asset {
   fn ncols(&self) -> usize { 6 }
   fn as_csv(&self) -> String {
      format!("{},{},{},{},{},{}", 
              self.date, self.blockchain, self.token, self.quote, self.amount,
              mk_usd(self.amount * self.quote.amount))
   }
}

impl Asset {
   pub fn key(&self) -> (Blockchain, Token) {
      (self.blockchain.clone(), self.token.clone())
   }
   pub fn tvl(&self) -> USD { mk_usd(self.amount * self.quote.amount) }
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


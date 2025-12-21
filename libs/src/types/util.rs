use std::cmp::Ordering;

use chrono::NaiveDate;

use book::{
   csv_utils::CsvWriter,
   currency::usd::{USD,mk_usd}
};

// ----- Your basic types used across all domains -------------------------

pub type Id = usize;
pub type Token = String;
pub type Pool = (Token, Token);

pub type Blockchain = String; // enum? maybe, but String for now.

// ----- CSV types --------------------------------------------------------

pub trait CsvHeader {
   fn header(&self) -> String;
}

// ----- PARTITION type ---------------------------------------------------

pub type Partition<T> = (Vec<T>, Vec<T>);

// ----- Measurable types -------------------------------------------------

pub trait Measurable {
   fn sz(&self) -> f32;
   fn aug(&self) -> f32;
}

pub fn size<T: Measurable>(v: &Vec<T>) -> f32 {
   v.iter().map(Measurable::sz).sum()
}

pub fn weight<T: Measurable>(v: &Vec<T>) -> f32 {
   let (au, s) =
      v.iter()
       .fold((0.0, 0.0), |(a,b), x| (a + x.aug(), b + x.sz()));
   au / s
}

pub fn sort_by_weight<M: Measurable>(a: &M, b: &M) -> Ordering {
   b.aug().total_cmp(&a.aug())
}

pub fn sort_by_size<M: Measurable>(a: &M, b: &M) -> Ordering {
   b.sz().total_cmp(&a.sz())
}

pub fn sort_descending<M: Measurable>(a: &M, b: &M) -> Ordering {
   sort_by_weight(a, b)
}

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

pub fn tvl(a: &Asset) -> USD { mk_usd(a.amount * a.quote.amount) }


use std::cmp::Ordering;

use chrono::NaiveDate;

use book::{
   csv_utils::CsvWriter,
   currency::usd::USD
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

pub fn sort_descending<M: Measurable>(a: &M, b: &M) -> Ordering {
   b.aug().total_cmp(&a.aug())
}

// ----- ASSETS ----------------------------------------------------------

/// An Asset (an element of Assets) is a Token distinguished by Blockchain
#[derive(Debug, Clone)]
pub struct Asset {
   blockchain: Blockchain,
   token: Token,
   amount: f32
   // quote: USD,
   // date: NaiveDate
}

/*
impl Measurable for Asset {
   fn sz(&self) -> f32 { self.amount }
   fn aug(&self) -> f32 { self.sz() * self.quote.amount }
}
*/

impl CsvHeader for Asset {
   fn header(&self) -> String {
      "for_pivot,blockchain,token,quote,amount,total".to_string()
   }
}
impl CsvWriter for Asset {
   fn ncols(&self) -> usize { 3 }
   fn as_csv(&self) -> String {
      format!("{},{},{}", self.blockchain, self.token, self.amount)
   }
}

fn addm(amt: f32, mult: i32, adj: Option<&f32>) -> f32 {
   let ans = adj.and_then(|a| Some(mult as f32 * a + amt));
   if let Some(x) = ans { x } else { amt }
}

impl Asset {
   pub fn key(&self) -> (Blockchain, Token) {
      (self.blockchain.clone(), self.token.clone())
   }
   pub fn madd(&self, amt: Option<&f32>) -> f32 { addm(self.amount, 1, amt) }
   pub fn msubtract(&self, amt: Option<&f32>) -> f32 { 
      addm(self.amount, -1, amt)
   }
}

pub fn mk_asset(k: &(Blockchain, Token), amount: f32) -> Asset {
   let (b, t) = k;
   Asset { blockchain: b.clone(), token: t.clone(), amount }
}


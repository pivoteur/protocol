use chrono::{Days,NaiveDate};

use book::{
   csv_utils::{CsvHeader,CsvWriter},
   currency::usd::{USD,mk_usd},
   err_utils::ErrStr,
   num::percentage::{Percentage,mk_percentage},
   tuple_utils::Partition,
   utils::pred
};

use crate::types::{
   quotes::Quotes,
   util::{Token,Blockchain,Id,Pool},
   measurable::{Measurable,weight,size}
};

// ----- ASSETTYPES

#[derive(Debug, Clone, PartialEq)]
pub enum AssetType { FROM, TO }
use AssetType::*;

impl AssetType {
   fn keys(&self) -> Vec<String> {
      match self {
         FROM =>
            slice2vec(&["from","from_blockchain","amount1","virtual","quote1"]),
         TO => slice2vec(&["to","to_blockchain","net","blah!","quote2"])
      }
   }
   fn kind(&self) -> String {
      match self {
         AssetType::FROM => "from".to_string(),
         AssetType::TO => "to".to_string()
      }
   }
   fn headers(&self) -> String {
      (if self == &FROM { "amount1,virtual" } else { "net,ersatz"}).to_string()
   }
   fn ix(&self) -> usize { if self == &FROM { 1 } else { 2 } }
}

fn slice2vec(ss: &[&str]) -> Vec<String> {
   let mut vec: Vec<String> = Vec::new();
   for s in ss {
      vec.push(s.to_string());
   }
   vec
}

pub fn kinderize(k: &AssetType, s: &[&str]) -> Vec<String> {
   s.iter().map(|elt| format!("{}{}", k.kind(), elt)).collect()
}


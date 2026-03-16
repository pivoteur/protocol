use chrono::NaiveDate;
use book::err_utils::ErrStr;
use crate::types::Coin::{Coin,mk_coin};
use super::assets::Asset;

#[derive(Debug, Clone, PartialEq)]
pub enum AssetType { FROM, TO }
use AssetType::*;

impl AssetType {
   pub fn keys(&self) -> Vec<String> {
      match self {
         FROM =>
            slice2vec(&["from","from_blockchain","amount1","virtual","quote1"]),
         TO => slice2vec(&["to","to_blockchain","net","blah!","quote2"])
      }
   }
   pub fn kind(&self) -> String {
      match self {
         AssetType::FROM => "from".to_string(),
         AssetType::TO => "to".to_string()
      }
   }
   pub fn headers(&self) -> String {
      (if self == &FROM { "amount1,virtual" } else { "net,ersatz"}).to_string()
   }
   pub fn ix(&self) -> usize { if self == &FROM { 1 } else { 2 } }

   pub fn committed(&self, asset: &Asset, dt: &NaiveDate) -> ErrStr<Coin> {
      match &self {
         &FROM =>
            Err("Computing committed amount from a FROM asset!".to_string()),
         &TO => {
            let blockchain = asset.blockchain();
            let token = asset.token();
            let src = &(blockchain, token);
            Ok(mk_coin(src, asset.sz(), &mk_usd(asset.aug()), dt))
         }
      }
   }
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


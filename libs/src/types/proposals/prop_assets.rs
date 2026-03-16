use chrono::NaiveDate;

use book::{
   csv_utils::{CsvHeader,CsvWriter},
   currency::usd::{USD,mk_usd}
};

use crate::types::{
   assets::asset_types::{AssetType,AssetType::*},
   util::{Token,Blockchain,Pool},
   measurable::{Measurable,weight,size},
   coins::{Coin,mk_coin}
};

#[derive(Debug, Clone)]
pub struct PropAsset {
   token: Token,
   blockchain: Blockchain,
   close_price: USD,
   amount: f32,
   kind: AssetType
}

impl PropAsset {
   pub fn clone_with(&self, amount: f32, kind: AssetType) {
      mk_prop_asset(&self.token, &self.blockchain, &self.close_price,
                    amount, kind)
   }
}

impl CsvHeader for PropAsset {
   fn header(&self) -> String {
      let preface = match self.kind {
         FROM => "pivot",
         TO   => "proposed"
      };
      ["token","blockchain","close_price","amount"]
         .iter()
         .map(|elt| format!("{}_{}", preface, elt))
         .collect::<Vec<_>>()
         .join(",")
   }
}
impl CsvWriter for PropAsset {
   fn ncols(&self) -> usize { 2 }
   fn as_csv(&self) -> String {
      format!("{},{},{},{}",
              self.token, self.blockchain, self.close_price, self.amount)
   }
}

impl Measurable for PropAsset {
   fn sz(&self) -> f32 { self.amount }
   fn aug(&self) -> f32 { self.sz() * self.close_price.amount }
}

pub fn mk_prop_asset(t: &str, b: &str, c: &USD, amount: f32, kind: AssetType)
      -> PropAsset {
   PropAsset { token: t.to_string(), blockchain: b.to_string(),
               close_price: c.clone(), amount, kind }
}

pub fn pivot_amount0(blockchain: Blockchain, pool: Pool,
                 date: &NaiveDate, assets: &Vec<PropAsset>) -> Coin {
   let (_, piv) = pool.clone();
   mk_coin(&(blockchain, piv), size(&assets), &mk_usd(weight(&assets)), &date)
}


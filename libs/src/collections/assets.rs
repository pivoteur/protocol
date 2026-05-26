use std::collections::HashMap;

use book::{
   csv_utils::{CsvWriter,CsvHeader},
   err_utils::ErrStr
};

use crate::types::{
   tokens::coins::Coin,
   measurable::{Measurable,sort_by_tvl,sort_by_weight},
   quotes::Quotes,
   util::{Token,Blockchain}
};

/// An Assets (a singular collection of a plurality of assets) is a bag
/// where the size is the amount of the asset

#[derive(Debug)]
pub struct Assets { map: HashMap<(Blockchain,Token), Coin> }

pub fn mk_assets() -> Assets { Assets { map: HashMap::new() } }

impl Assets {
   pub fn add(&mut self, asset: Coin) {
      self.map.entry(asset.key())
              .and_modify(|a| { *a += asset.sz(); })
              .or_insert(asset);
   }
   pub fn subtract(&mut self, asset: &Coin) {
      let k = asset.key();
      if let Some(a) = self.map.get_mut(&k) {
         let sub = asset.sz();
         let amt = a.sz() - sub;
         if amt < 0.0 { panic!("Cannot have a negative amount of {}
Trying to subtract this amount: {sub}
Coin: {}

assets:
{}", a.as_csv(), asset.as_csv(), self.as_csv());
         }
         if amt == 0.0 { self.map.remove(&k); } else { *a += -sub; }
      } else {
         panic!("No asset {:?} to remove!", asset)
      }
   }
   pub fn update_prices(&mut self, qs: &Quotes) -> ErrStr<()> {
      for coin in self.map.values_mut() {
         coin.update_price(qs)?;
      }
      Ok(())
   }
   pub fn is_empty(&self) -> bool { self.map.is_empty() }
   pub fn assets(&self) -> Vec<Coin> { self.map.values().cloned().collect() }
}

impl CsvHeader for Assets { 
   fn header(&self) -> String {
      let proto = Coin::default();
      proto.header()
   }
}

impl CsvWriter for Assets {
   fn ncols(&self) -> usize {
      let proto = Coin::default();
      proto.ncols()
   }
   fn as_csv(&self) -> String {
      let ans: Vec<String> =
         self.map.values().map(CsvWriter::as_csv).collect();
      ans.join("\n")
   }
}

pub fn assets_by_price(a: &Assets) -> Vec<Coin> {
   let mut ans = a.assets();
   ans.sort_by(sort_by_weight);
   ans
}

pub fn assets_by_tvl(a: &Assets) -> Vec<Coin> {
   let mut ans = a.assets();
   ans.sort_by(sort_by_tvl);
   ans
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, err_utils::ErrStr };
   use crate::types::tokens::coins::functional_tests::coin;

   create_testing!("types::pivots");

   run!("mk_assets_and_add", {
      let btc = coin("BTC", 1.0)?;
      let eth = coin("ETH", 34.0)?;
      let mut assets = mk_assets();
      for asset in [btc, eth] { assets.add(asset); }
      println!("\tAssets with BTC and ETH:\n\n{}", assets.as_csv());
   });
}


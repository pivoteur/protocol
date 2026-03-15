use std::collections::HashMap;

use crate::types::{
   coins::Coin,
   measurable::{sort_by_tvl,sort_by_weight},
   util::{Token,Blockchain}
};

/// An Assets (a singular collection of a plurality of assets) is a bag
/// where the size is the amount of the asset

pub struct Assets { map: HashMap<(Blockchain,Token), Coin> }

pub fn mk_assets() -> Assets { Assets { map: HashMap::new() } }

impl Assets {
   pub fn add(&mut self, asset: Coin) {
      self.map.entry(asset.key())
          .and_modify(|a| {
             a.amount += asset.amount;
             if asset.date > a.date {
                a.quote.amount = asset.quote.amount;
                a.date = asset.date;
             }
          })
          .or_insert(asset);
   }
   pub fn subtract(&mut self, asset: &Coin) {
      let k = asset.key();
      if let Some(a) = self.map.get_mut(&k) {
         let amt = a.amount - asset.amount;
         if amt < 0.0 { panic!("Withdrew more {:?} than available!", a); }
         if amt == 0.0 { self.map.remove(&k); } else { a.amount = amt; }
      } else {
         panic!("No asset {:?} to remove!", asset)
      }
   }
   pub fn is_empty(&self) -> bool { self.map.is_empty() }
}

fn assets(a: &Assets) -> Vec<Coin> { a.map.values().cloned().collect() }

pub fn assets_by_price(a: &Assets) -> Vec<Coin> {
   let mut ans = assets(a);
   ans.sort_by(sort_by_weight);
   ans
}

pub fn assets_by_tvl(a: &Assets) -> Vec<Coin> {
   let mut ans = assets(a);
   ans.sort_by(sort_by_tvl);
   ans
}


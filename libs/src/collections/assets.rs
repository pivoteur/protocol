use std::collections::HashMap;

use crate::types::{
   assets::Asset,
   measurable::{sort_by_weight},
   util::{Token,Blockchain}
};

/// An Assets (a singular collection of a plurality of assets) is a bag
/// where the size is the amount of the asset

pub struct Assets { map: HashMap<(Blockchain,Token), Asset> }

pub fn mk_assets() -> Assets { Assets { map: HashMap::new() } }

impl Assets {
   pub fn add(&mut self, asset: Asset) {
      self.map.entry(asset.key())
          .and_modify(|a| a.amount += asset.amount)
          .or_insert(asset);
   }
   pub fn subtract(&mut self, asset: &Asset) {
      let k = asset.key();
      if let Some(a) = self.map.get_mut(&k) {
         let amt = a.amount - asset.amount;
         if amt < 0.0 { panic!("Withdrew more {:?} than available!", a); }
         if amt == 0.0 { self.map.remove(&k); } else { a.amount = amt; }
      } else {
         panic!("No asset {:?} to remove!", asset)
      }
   }
   pub fn assets(&self) -> Vec<Asset> {
      let mut ans: Vec<Asset> = self.map.values().cloned().collect();
      ans.sort_by(sort_by_weight);
      ans
   }
}

pub fn assets_by_price(a: &Assets) -> Vec<Asset> { a.assets() }
pub fn assets_by_tvl(a: &Assets) -> Vec<Asset> {
   let mut ans = a.assets();
   ans.sort_by(|a, b| b.tvl().cmp(&a.tvl()));
   ans
}

/// One way to look at a PivotPool is that it is an assets, ... I mean:
/// I had pivot pools with three assets before ... it wasn't a good idea then,
/// but who's to say I won't reevaluate that decision?

pub type PivotPool = Assets;


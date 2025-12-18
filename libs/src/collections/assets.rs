use std::collections::HashMap;

use crate::types::util::{Token, Blockchain,Asset,mk_asset};

/// An Assets (a singular collection of a plurality of assets) is a bag
/// where the size is the amount of the asset

pub type Assets = HashMap<(Blockchain,Token), f32>;

pub fn add(m: &mut Assets, a: &Asset) {
   let k = a.key();
   let amt = a.madd(m.get(&k));
   m.insert(k, amt);
}
pub fn subtract(m: &mut Assets, a: &Asset) {
   let k = a.key();
   let amt = a.msubtract(m.get(&k));
   if amt < 0.0 { panic!("Withdrew more {:?} than available!", a); }
   if amt == 0.0 { m.remove(&k); } else { m.insert(k, amt); }
}

pub fn assets(a: &Assets) -> Vec<Asset> {
   let mut v = Vec::new();
   for (k,amt) in a {
      v.push(mk_asset(k, *amt));
   }
   v
}


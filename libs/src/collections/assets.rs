use std::collections::HashMap;

use book::{
   csv_utils::{CsvWriter,CsvHeader},
   currency::usd::mk_usd,
   err_utils::ErrStr,
   string_utils::s
};

use crate::types::{
   comps::{ Composition, mk_composition },
   tokens::coins::{ Coin, mk_coin },
   measurable::{Measurable,sort_by_tvl,sort_by_weight},
   pools::Pool,
   quotes::Quotes,
   util::{Token,Blockchain}
};

/// An Assets (a singular collection of a plurality of assets) is a bag
/// where the size is the amount of the asset

#[derive(Debug)]
pub struct Assets { map: HashMap<(Blockchain,Token), Coin> }

pub fn mk_assets() -> Assets { Assets { map: HashMap::new() } }

impl Assets {
   pub fn brief(&self) -> String {
      self.map.iter()
          .map(|((_,tok), coin)| format!("{} {tok}", coin.sz()))
          .collect::<Vec<_>>()
          .join(", ")
   }
   pub fn add(&mut self, asset: Coin) {
      self.map.entry(asset.key())
              .and_modify(|a| { *a += asset.sz(); })
              .or_insert(asset);
   }
   pub fn subtract(&mut self, asset: &Coin) {
      let k = asset.key();
      let (_, tok) = &k;
      if let Some(a) = self.map.get_mut(&k) {
         let sub = asset.sz();
         let amt = a.sz() - sub;
         if amt < 0.0 {
            panic!("
Cannot have a negative amount of {tok}

Trying to subtract this amount: {sub} {tok}
from assets {}
", self.brief())
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
   pub fn as_composition(&mut self, pool: &Pool, quotes: &Quotes)
         -> ErrStr<Composition> {
/* 4 scenarii:

1. no matches, no virtual pivots
2. 1 match on primary
3. 1 match on pivot
4. 2 matches: primary, pivot

so, you know: handle those.
*/

      let default_blockchain = s("Avalanche");
      let blk =
         self.map.keys()
                 .next()
                 .and_then(|(b,_)| Some(b))
                 .or(Some(&default_blockchain))
                 .unwrap()
                 .clone();
      fn nonce<'a>(b: &'a Blockchain, q: &'a Quotes)
            -> impl Fn(&'a Token) -> ErrStr<Coin> {
         move |tok| {
            let qt = q.lookup(tok)?;
            Ok(mk_coin(&(b.clone(), tok.clone()), 0.0, &mk_usd(qt), &q.date))
         }
      }
      let (pri, piv) = pool.as_tuple();
      let zed = nonce(&blk, &quotes);
      self.add(zed(&pri)?);
      self.add(zed(&piv)?);
      let abp = assets_by_price(&self);
      if let [pr, pv] = abp.as_slice() {
         Ok(mk_composition(pr, pv))
      } else {
         Err(format!("Cannot create a composition from {}", self.as_csv()))
      }
   }
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
mod test_data {
   use super::*;
   use crate::types::tokens::coins::{ Coin, test_data::coin };

   pub fn test_btc_coin() -> ErrStr<Coin> { coin("BTC", 1.0) }
   pub fn test_eth_coin() -> ErrStr<Coin> { coin("ETH", 34.0) }
   pub fn test_btc_eth_assets() -> ErrStr<Assets> {
      let mut assets = mk_assets();
      for asset in [test_btc_coin(), test_eth_coin()] { assets.add(asset?); }
      Ok(assets)
   }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use super::test_data::test_btc_eth_assets;
   use paste::paste;
   use book::{ create_testing, err_utils::ErrStr };

   create_testing!("types::pivots");

   run!("mk_assets_and_add", {
      let assets = test_btc_eth_assets()?;
      println!("\tAssets with BTC and ETH:\n\n{}", assets.as_csv());
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use super::test_data::{ test_btc_coin, test_eth_coin, test_btc_eth_assets };
   use crate::types::{
      pools::pool_from_str,
      quotes::sample_data::sample_btc_eth_quotes,
      tokens::coins::test_data::coin
   };

   fn mk_sample_btc_eth_composition(assets: &mut Assets)
         -> ErrStr<Composition> {
      let quotes = sample_btc_eth_quotes();
      let pool = pool_from_str("btc-eth")?;
      assets.as_composition(&pool, &quotes)
   }

   fn comp_ok(assets: &mut Assets) -> ErrStr<()> {
      let comp = mk_sample_btc_eth_composition(assets);
      assert!(comp.is_ok());
      Ok(())
   }

   #[test] fn test_no_assets_composition_ok() -> ErrStr<()> {
      let mut assets = test_btc_eth_assets()?;
      assets.subtract(&test_btc_coin()?);
      assets.subtract(&test_eth_coin()?);
      assert!(assets.map.is_empty());
      comp_ok(&mut assets)
   }

   #[test] fn test_one_asset_composition_ok() -> ErrStr<()> {
      let mut assets = test_btc_eth_assets()?;
      assets.subtract(&test_btc_coin()?);
      assert_eq!(1, assets.map.len());
      comp_ok(&mut assets)
   }

   #[test] fn test_two_assets_composition_ok() -> ErrStr<()> {
      let mut assets = test_btc_eth_assets()?;
      assert_eq!(2, assets.map.len());
      comp_ok(&mut assets)
   }

   #[test] fn fail_three_assets_composition() -> ErrStr<()> {
      let mut assets = test_btc_eth_assets()?;
      assets.add(coin("USDC", 72043.0)?);
      assert_eq!(3, assets.map.len());
      let comp = mk_sample_btc_eth_composition(&mut assets);
      assert!(comp.is_err());
      Ok(())
   }
}

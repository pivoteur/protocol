use chrono::NaiveDate;

use book::{
   currency::usd::{USD,mk_usd},
   csv_utils::{CsvHeader,CsvWriter},
   err_utils::ErrStr,
   string_utils::plural,
};

use crate::collections::assets::{Assets,mk_assets};

use super::{
   tokens::coins::{Coin,PivotCoin},
   measurable::{Measurable,size,tvl},
   util::pool_name as pool_nm
};

#[derive(Debug,Clone)]
pub struct Composition {
   primary: Coin,
   pivot: PivotCoin
}

#[cfg(test)]
mod test_data {
   use book::err_utils::ErrStr;
   use super::{ Composition, mk_composition, from_assets };
   use crate::{
      collections::assets::mk_assets,
      types::tokens::coins::functional_tests::coin
   };
   
   pub fn mk_btc_eth() -> ErrStr<Composition> {
      let btc = coin("BTC", 0.1)?;
      let eth = coin("ETH", 3.4)?;
      Ok(mk_composition(&eth, &btc))
   }
   pub fn mk_undead_usdc() -> ErrStr<Composition> {
      let mut assets = mk_assets();
      assets.add(coin("UNDEAD", 1000000.0)?);
      assets.add(coin("USDC", 1400.0)?);
      from_assets(&assets.assets())
   }
}

mod asset_ordering {
   use super::Composition;
   use crate::types::{
      tokens::coins::{Coin,mk_pivot_coin},
      measurable::Measurable
   };
   use book::{ num::floats::mk_safe_float, tuple_utils::{ Partition, snd } };

   fn sort_asset_pair<'a>(a: &'a Coin, b: &'a Coin) -> (&'a Coin, &'a Coin) {
      let mut coins = vec![a, b];
      coins.sort_by_key(|f| mk_safe_float(&f.aug()));
      (coins.pop().unwrap(), coins.pop().unwrap())
   }

   pub fn arrange_assets(a: &Coin, b: &Coin) -> Composition {
      let (c, d): Partition<Coin> =
         [a.clone(), b.clone()].into_iter()
                   .partition(|c| snd(c.key()).contains("USD"));
      let (x, y) = if c.is_empty() {
         sort_asset_pair(a, b)
      } else {
         // this does not cover the case of a USD/USD pair (e.g.: USDC/USDt)
         (d.first().unwrap(), c.first().unwrap())
      };
      mk_composition0(x, y)
   }

   fn mk_composition0(x: &Coin, y: &Coin) -> Composition {
      Composition { primary: x.clone(), pivot: mk_pivot_coin(y.clone()) }
   }

   #[cfg(test)]
   mod tests {
      use super::*;
      use crate::types::{ comps::test_data::*, util::{ Pool, pool_from_str } };
      use book::err_utils::ErrStr;

      fn assert_pool_tokens<'a>(pool: Pool) -> impl Fn(&'a Coin, &'a Coin) {
         move |a: &'a Coin, b: &'a Coin| {
            assert_eq!(pool.0, snd(a.key()));
            assert_eq!(pool.1, snd(b.key()));
         }
      }

      #[test] fn test_sort_asset_pair_undead_usdc() -> ErrStr<()> {
         let uu = mk_undead_usdc()?;
         let piv = &uu.pivot.coin();
         let (a, b) = sort_asset_pair(&uu.primary, piv);
         assert_pool_tokens(pool_from_str("undead-usdc")?)(b, a);
         Ok(())
      }

      #[test] fn test_arrange_assets_undead_usdc() -> ErrStr<()> {
         let uu = mk_undead_usdc()?;
         let piv = &uu.pivot.coin();
         let comp = arrange_assets(&uu.primary, piv);
         let piv_coin = comp.pivot.coin();
         assert_pool_tokens(pool_from_str("undead-usdc")?)
                           (&comp.primary, &piv_coin);
         Ok(())
      }

      #[test] fn test_btc_eth_arranging_and_sorting() -> ErrStr<()> {
         let itbe = mk_btc_eth()?;
         let btc = &itbe.primary;
         let eth = &itbe.pivot.coin();
         let (b, e) = sort_asset_pair(&eth, &btc);
         let comp = arrange_assets(&eth, &btc);
         let piv = comp.pivot.coin();
         let assert_btc_eth = assert_pool_tokens(pool_from_str("btc-eth")?);
         assert_btc_eth(&btc, &eth);
         assert_btc_eth(b, e);
         assert_btc_eth(&comp.primary, &piv);
         Ok(())
      }
   }
}

use asset_ordering::arrange_assets;

pub fn mk_composition(primary: &Coin, pivot: &Coin) -> Composition {
   arrange_assets(primary, pivot)
}

pub fn from_assets(assets: &[Coin]) -> ErrStr<Composition> {
   match assets {
      [a, b] => Ok(arrange_assets(&a, &b)),
      _ => Err(format!("Cannot make a composition from {}",
                       plural(assets.len(), "asset")))
   }
}

impl Composition {
   pub fn pool_name(&self) -> String { 
      let (_, pri) = self.primary.key();
      let piv = self.pivot.key();
      pool_nm(&(pri, piv))
   }

   pub fn tvl(&self) -> USD { tvl(&self.primary) + tvl(&self.pivot) }
   pub fn as_assets(&self) -> Assets {
      let mut assets = mk_assets();
      assets.add(self.primary.clone());
      assets.add(self.pivot.coin());
      assets
   }
}

impl Measurable for Composition {
   fn sz(&self) -> f32 { self.tvl().amount() }
   fn aug(&self) -> f32 { 1.0 }
}

pub fn total(pools: &Vec<Composition>) -> USD {
   mk_usd(size(pools))
}

pub fn last_updated(pools: &Vec<Composition>) -> Option<NaiveDate> {
   pools.iter().map(|p| p.primary.date).max()
}

impl CsvWriter for Composition {
   fn ncols(&self) -> usize {
      1 + self.primary.ncols() + self.pivot.ncols() + 1
   }
   fn as_csv(&self) -> String {
      format!("{},{},{},{}",
              self.pool_name(),
              self.primary.as_csv(),
              self.pivot.as_csv(),
              self.tvl())
   }
}

impl CsvHeader for Composition {
   fn header(&self) -> String {
      format!("pool,{},{},tvl",
              contextualize(PRIMARY, &self.primary.header()),
              contextualize(PIVOT, &self.pivot.header()))
   }
}

enum PoolCoin { PRIMARY, PIVOT }
use PoolCoin::*;

impl PoolCoin {
   fn kind(&self) -> String {
      match self {
         PRIMARY => "primary",
         PIVOT => "pivot"
      }.to_string()
   }
}

fn contextualize(p: PoolCoin, hdr: &str) -> String {
   hdr.split(",").map(|s| format!("{}_{}", p.kind(), s))
      .collect::<Vec<_>>()
      .join(",")
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use super::test_data::*;
   use paste::paste;
   use book::create_testing;
   use crate::types::tokens::coins::functional_tests::coin;

   create_testing!("types::comps");

   run_with!("mk_composition", " (BTC+ETH pivot pool)",
             &mk_btc_eth()?, CsvWriter::as_csv);
   run_with!("as_assets", " (BTC+ETH pivot pool)",
             &mk_btc_eth()?.as_assets(), CsvWriter::as_csv);
   run_with!("from_assets_undead_usdc", &mk_undead_usdc()?, CsvWriter::as_csv);
   run!("from_assets_btc_usdc", " (BTC and USDC assets)", {
      let mut assets = mk_assets();
      assets.add(coin("BTC", 0.1)?);
      assets.add(coin("USDC", 8500.0)?);
      let comp = from_assets(&assets.assets())?;
      println!("\tBTC+USDC assets:\n{}", comp.as_csv());
   });

   mod tests {
      use super::*;
      use book::tuple_utils::snd;

      #[test] fn fail_from_0_assets() {
         let ans = from_assets(&mk_assets().assets());
         assert!(ans.is_err());
      }
      #[test] fn fail_from_too_many_assets() -> ErrStr<()> {
         let mut assets = mk_assets();
         assets.add(coin("AVAX", 12.0)?);
         assets.add(coin("BTC", 0.1)?);
         assets.add(coin("USDC", 100.0)?);
         let ans = from_assets(&assets.assets());
         assert!(ans.is_err());
         Ok(())
      }
      #[test] fn test_from_undead_usdc_assets() -> ErrStr<()> {
         let undead_usdc = mk_undead_usdc()?;
         assert_eq!("UNDEAD", &snd(undead_usdc.primary.key()));
         assert_eq!("USDC", &undead_usdc.pivot.key());
         Ok(())
      }
      #[test] fn test_mk_btc_eth_composition() -> ErrStr<()> {
         let btc_eth = mk_btc_eth()?;
         assert_eq!("BTC", &snd(btc_eth.primary.key()));
         assert_eq!("ETH", btc_eth.pivot.key());
         Ok(())
      }
   }
}


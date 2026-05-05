use chrono::NaiveDate;

use book::{
   currency::usd::{USD,mk_usd},
   csv_utils::{CsvHeader,CsvWriter}
};

use crate::collections::assets::{Assets,mk_assets};
use super::{
   coins::{Coin,PivotCoin,mk_pivot_coin},
   measurable::{Measurable,size,tvl},
   util::pool_name as pool_nm
};

#[derive(Debug,Clone)]
pub struct Composition {
   primary: Coin,
   pivot: PivotCoin
}

pub fn mk_composition(primary: Coin, pivot: Coin) -> Composition {
   Composition { primary, pivot: mk_pivot_coin(pivot) }
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
   fn sz(&self) -> f32 { self.tvl().amount }
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
   use paste::paste;
   use book::{ create_testing, err_utils::ErrStr };
   use crate::types::coins::functional_tests::coin;

   create_testing!("types::comps");

   fn mk_btc_eth() -> ErrStr<Composition> {
      let btc = coin("BTC", 0.1)?;
      let eth = coin("ETH", 3.4)?;
      Ok(mk_composition(btc, eth))
   }

   run_with!("mk_composition", " (BTC+ETH pivot pool)",
             &mk_btc_eth()?, CsvWriter::as_csv);
   run_with!("as_assets", " (BTC+ETH pivot pool)",
             &mk_btc_eth()?.as_assets(), CsvWriter::as_csv);

   run_all_functional_tests!();
}


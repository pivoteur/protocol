use chrono::NaiveDate;

use book::{
   currency::usd::{USD,mk_usd},
   csv_utils::CsvWriter
};

use super::{
   assets::{Asset,PivotAsset,tvl,mk_pivot_asset},
   measurable::{Measurable,size},
   util::CsvHeader
};

#[derive(Debug,Clone)]
pub struct Composition {
   primary: Asset,
   pivot: PivotAsset
}

pub fn mk_composition(primary: Asset, pivot: Asset) -> Composition {
   Composition { primary, pivot: mk_pivot_asset(pivot) }
}

impl Composition {
   pub fn pool_name(&self) -> String {
      let (_, pri) = self.primary.key();
      let piv = self.pivot.key();
      format!("{pri}+{piv}")
   }

   pub fn tvl(&self) -> USD { tvl(&self.primary) + tvl(&self.pivot) }
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

enum PoolAsset { PRIMARY, PIVOT }
use PoolAsset::*;

impl PoolAsset {
   fn kind(&self) -> String {
      match self {
         PRIMARY => "primary",
         PIVOT => "pivot"
      }.to_string()
   }
}

fn contextualize(p: PoolAsset, hdr: &str) -> String {
   hdr.split(",").map(|s| format!("{}_{}", p.kind(), s))
      .collect::<Vec<_>>()
      .join(",")
}


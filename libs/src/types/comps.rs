use book::{
   currency::usd::USD,
   csv_utils::CsvWriter
};

use crate::types::{
   assets::Asset,
   util::CsvHeader
};

#[derive(Debug,Clone)]
pub struct Composition {
   primary: Asset,
   pivot: Asset
}

pub fn mk_composition(primary: Asset, pivot: Asset) -> Composition {
   Composition { primary, pivot }
}

impl Composition {
   pub fn pool_name(&self) -> String {
      let (_, pri) = self.primary.key();
      let (_, piv) = self.pivot.key();
      format!("{pri}+{piv}")
   }

   pub fn tvl(&self) -> USD { self.primary.tvl() + self.pivot.tvl() }
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
              contextualize(PoolType::PRIMARY, &self.primary.header()),
              contextualize(PoolType::PIVOT, &self.pivot.header()))
   }
}

enum PoolType { PRIMARY, PIVOT }
use PoolType::*;

impl PoolType {
   fn kind(&self) -> String {
      match self {
         PRIMARY => "primary",
         PIVOT => "pivot"
      }.to_string()
   }
}

fn contextualize(p: PoolType, hdr: &str) -> String {
   hdr.split(",").map(|s| format!("{}_{}", p.kind(), s))
      .collect::<Vec<_>>()
      .join(",")
}


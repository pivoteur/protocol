use chrono::NaiveDate;

use serde::Serialize;
use serde_with::{ serde_as, DisplayFromStr };

use book::{
   not_implemented,
   err_utils::ErrStr,
   json_utils::AsJSON,
   num::percentage::Percentage
};
use crate::types::{
   pivots::opens::Pivot,
   quotes::Quotes
};
use super::{ pool_assets::PoolAssets, pool_names::PoolName };

#[derive(Debug, Clone)]
pub struct PoolHealth {
   name: PoolName,
   assets: Vec<PoolAssets>,
   open_pivots: Vec<Pivot>
}

impl PoolHealth {
   fn transform(&self) -> Health1 {
      not_implemented!("PoolHealth::transform", self)
   }
}

impl AsJSON for PoolHealth {
   fn as_json(&self) -> String {
      let h1 = &self.transform();
      match serde_json::to_string(&h1) {
         Ok(json_string) => format!("{}", json_string),
         Err(e) => panic!("Failed to serialize: {}", e),
      }
   }
}

#[serde_as]
#[derive(Debug, Clone, Serialize)]
struct Health1 {
   #[serde_as(as = "DisplayFromStr")]
   pool: PoolName,
   #[serde_as(as = "DisplayFromStr")]
   incept: NaiveDate,
   // #[serde_as(as = "DisplayFromStr")]
   // tvl: TVL,
   #[serde_as(as = "DisplayFromStr")]
   roi: Percentage,
   #[serde_as(as = "DisplayFromStr")]
   apr: Percentage
}
   
pub fn mk_pool_health(q: &Quotes, n: &PoolName, a: &[PoolAssets], v: &[Pivot])
      -> ErrStr<PoolHealth> {
   not_implemented!("pool_health", q, n, a, v)
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::create_testing;
   use crate::{
      fetchers::pivots::sample_reader::read_sample_open_pivots,
      types::{
         pools::{
            pool_assets::sample_data::sample_btc_eth_pool_assets,
            pool_names::pool_name_from_str
         },
         quotes::sample_data::sample_btc_eth_quotes
      }
   };

   create_testing!("types::pools::health");

   run!("mk_pool_health", {
      let pool = "btc-eth";
      let quiz = "../quizzes";
      let q = sample_btc_eth_quotes();
      let name = pool_name_from_str(pool)?;
      let assets = sample_btc_eth_pool_assets(quiz)?;
      let filename = format!("{quiz}/data/sample_btc_eth_open_pivots.tsv");
      let open_pivots = read_sample_open_pivots(&filename, pool)?;
      let health = mk_pool_health(&q, &name, &assets, &open_pivots)?;
      println!("Pool health is:\n{}", health.as_json());
   });
}


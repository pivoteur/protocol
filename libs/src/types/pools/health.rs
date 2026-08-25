use chrono::NaiveDate;

use serde::Serialize;
use serde_with::{ serde_as, DisplayFromStr };

use book::{
   json_utils::AsJSON,
   num::percentage::Percentage
};
use crate::types::gains::Gains;
use super::{
   pool_assets::{ PoolAssets, incept, prototype },
   pool_names::PoolName,
   tvl::TVL
};

#[derive(Debug, Clone)]
pub struct PoolHealth {
   name: PoolName,
   assets: Vec<PoolAssets>,
   tvl: TVL
}

pub fn mk_pool_health(n: &PoolName, a: &[PoolAssets], t: &TVL) -> PoolHealth {
   PoolHealth {
      name: n.clone(),
      assets: a.to_vec(),
      tvl: t.clone()
   }
}

impl PoolHealth {
   fn transform(&self) -> Option<Health1> {
      prototype(&self.assets).and_then(|pa|
         incept(&self.assets).and_then(| incept | {
            let pool = self.name.clone();
            let roi = pa.roi();
            let apr = pa.apr();
            Some(Health1 { pool, incept, roi, apr, tvl: self.tvl.clone() })
         })
      )
   }
}

impl AsJSON for PoolHealth {
   fn as_json(&self) -> String {
      let h1 = &self.transform();
      match serde_json::to_string_pretty(&h1) {
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
   tvl: TVL,
   #[serde_as(as = "DisplayFromStr")]
   roi: Percentage,
   #[serde_as(as = "DisplayFromStr")]
   apr: Percentage
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, err_utils::ErrStr };
   use crate::{
      types::{
         pools::{
            pool_assets::sample_data::sample_btc_eth_pool_assets,
            pool_names::pool_name_from_str,
            tvl::sample_data::sample_btc_eth_tvl
         }
      }
   };

   create_testing!("types::pools::health");

   run!("mk_pool_health", {
      let pool = "btc-eth";
      let offset = "../quizzes";
      let name = pool_name_from_str(pool)?;
      let assets = sample_btc_eth_pool_assets(offset)?;
      let tvl = sample_btc_eth_tvl();
      let health = mk_pool_health(&name, &assets, &tvl);
      println!("Pool health is:\n{}", health.as_json());
   });
}


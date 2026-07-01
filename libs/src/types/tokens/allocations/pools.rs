use chrono::NaiveDate;

use serde::{ Serialize,Serializer, ser::SerializeStruct };

use super::pool_assets::PoolAssets;
use crate::types::pools::Pool;
use book::utils::{ composer, deref };

#[derive(Debug,Clone)]
pub struct Pools {
   generated: NaiveDate,
   pools: Vec<PoolAssets>
}

pub fn mk_pools(dt: &NaiveDate, pools: Vec<PoolAssets>) -> Pools {
   Pools { generated: dt.clone(), pools }
}

impl Serialize for Pools {
   fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
         where S: Serializer {
      let mut state = serializer.serialize_struct("Pools", 3)?;

      state.serialize_field("generated", &format!("{}", self.generated))?;
      state.serialize_field("pools", &self.pools)?;

      let assets: Vec<Vec<String>> =
         self.pools.iter()
                   .map(composer(deref(Pool::as_vec), PoolAssets::as_pool))
                   .collect();
      state.serialize_field("assets", &assets)?;

      state.end()
   }
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod sample_pools {
   use super::{ Pools, mk_pools };
   use crate::types::tokens::allocations::pool_assets::test_data::sample_pool;
   use book::{ date_utils::yesterday, err_utils::ErrStr };

   // based off the pivot protocol, 2026-06-22
   pub fn sample_pools() -> ErrStr<Pools> {
      Ok(mk_pools(&yesterday(),
               vec![sample_pool("BTC", 0.29, "eth", 36.9)?,
                    sample_pool("btc", 0.06, "avax", 2389.3)?,
                    sample_pool("btc", 0.24, "usdc", 37021.35)?,
                    sample_pool("btc", 0.04, "undead", 22150170.0)?,
                    sample_pool("eth", 4.13, "undead", 16832550.0)?,
                    sample_pool("UNDEad", 18649824.0, "usdc", 6476.86)?,
                    sample_pool("avax", 792.56, "undead", 1890895.0)?]))
   }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::sample_pools::sample_pools;
   use paste::paste;
   use book::{ create_testing, err_utils::{ ErrStr, err_or } };

   create_testing!("types::tokens::allocations::pools");

   run!("deserialize_pools", {
      let pools = sample_pools()?;
      let json = err_or(serde_json::to_string_pretty(&pools),
              "Cannot serialize Pools assets")?;
      println!("The JSON of {pools:?} is\n\n{json}");
   });
}

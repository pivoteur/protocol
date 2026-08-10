use serde::Serialize;

use super::allocations::Allocation;

use crate::types::pools::{ Pool, mk_pool };

#[derive(Debug, Clone, Serialize)]
pub struct PoolAssets {
   primary: Allocation,
   pivot: Allocation
}

pub fn mk_pool_assets(primary: Allocation, pivot: Allocation) -> PoolAssets {
   PoolAssets { primary, pivot }
}

impl PoolAssets {
   pub fn as_pool(&self) -> Pool {
      mk_pool(&self.primary.key(), &self.pivot.key())
   }
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod test_data {
   use rand::Rng;
   use super::*;
   use crate::types::tokens::allocations::allocations::{
      Allocation,
      test_data::sample_allocation
   };
   use book::err_utils::ErrStr;

   fn rnd_alloc(tok: &str, alloc: f32) -> ErrStr<Allocation> {
      let virt: f32 = rand::thread_rng().gen_range(0.0 .. alloc);
      let act = alloc - virt;
      let ans = sample_allocation(&tok.to_uppercase(), virt, act)?;
      Ok(ans)
   }
   pub fn sample_pool(prim: &str, alloc_a: f32, piv: &str, alloc_b: f32)
         -> ErrStr<PoolAssets> {
      Ok(mk_pool_assets(rnd_alloc(prim, alloc_a)?, rnd_alloc(piv, alloc_b)?))
   }
}
      
#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::test_data::sample_pool;
   use paste::paste;
   use serde_json;

   use book::{ create_testing, err_utils::{ err_or, ErrStr } };

   create_testing!("types::tokens::allocations::pool_assets");

   run!("serialize_pool_assets", {
      let pool = sample_pool("btc", 0.5, "eth", 11.0)?;
      let json = err_or(serde_json::to_string_pretty(&pool),
                        "Cannot serizalize PoolAssets to JSON")?;
      println!("{pool:?} as JSON is:\n\n{json}");
   });
}

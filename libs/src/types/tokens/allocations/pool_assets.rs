/*
use chrono::NaiveDate;
use serde::{Serialize, ser::SerializeStruct, Serializer};
use serde_json;

use crate::types::{ measurable::Measurable, pools::Pool, util::Token };

use book::{ compose, err_utils::{ErrStr, err_or} };
*/

use serde::Serialize;

use super::allocations::Allocation;

// pub struct Pools 
   // quotes: Quotes,

#[derive(Debug, Clone, Serialize)]
pub struct PoolAssets {
   primary: Allocation,
   pivot: Allocation
}

pub fn mk_pool_assets(primary: Allocation, pivot: Allocation) -> PoolAssets {
   PoolAssets { primary, pivot }
}

/*
impl Serialize for PoolAssets {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("PoolAssets", 3)?;
        
        state.serialize_field("width", &self.width)?;
        state.serialize_field("height", &self.height)?;
        
        // Compute and inject the virtual field on the fly
        let area = self.width * self.height;
        state.serialize_field("area", &area)?;
        
        state.end()
    }
}

fn as_pool(p: &PoolAssets) -> Pool { mk_pool(&p.primary.token, &p.pivot.token) }

fn as_assets(pools: &[PoolAssets]) -> String {
   fn pool2pool(p: Pool) -> String {
      let (a, b) = p.as_tuple();
      format!("[ '{a}', '{b}' ]")
   }
   let assets: Vec<String> =
      pools.iter().map(compose!(pool2pool)(as_pool)).collect();
      format!("assets: [
      {}
   ]",  assets.join(",\n      "))
}

pub fn as_json(dt: &NaiveDate, v: &[PoolAssets]) -> ErrStr<String> {
   let assets = as_assets(v);
   let hdr = format!("const poolAssets = {
   generated: '{dt}',
   {assets},
   allocations = [
      {allocs}
   ]
serde_json::to_string_pretty(&user)
*/

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use crate::types::tokens::allocations::allocations::test_data::sample_allocation;
   use paste::paste;
   use serde_json;

   use book::{ create_testing, err_utils::{ err_or, ErrStr } };

   create_testing!("types::tokens::allocations::pool_assets");

   run!("serialize_pool_assets", {
      let btc = sample_allocation("BTC", 0.3, 0.1)?;
      let eth = sample_allocation("ETH", 2.2, 9.8)?;
      let pa = mk_pool_assets(btc, eth);
      let json = err_or(serde_json::to_string_pretty(&pa),
                        "Cannot serizalize PoolAssets to JSON")?;
      println!("{pa:?} as JSON is:\n\n{json}");
   });
}

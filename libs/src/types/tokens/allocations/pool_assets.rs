use chrono::NaiveDate;
use serde::{Serialize, ser::SerializeStruct, Serializer};
use serde_json;

use crate::types::{ measurable::Measurable, pools::Pool, util::Token };

use book::{ compose, err_utils::{ErrStr, err_or} };

#[derive(Debug, Clone)]
pub struct PoolAssets {
   generated: NaiveDate,
   primary: Allocation,
}

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

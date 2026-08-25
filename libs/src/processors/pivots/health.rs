use book::debug;

use crate::types::{
   measurable::Measurable,
   pivots::opens::Pivot,
   pools::{
      comps::Composition,
      health::{ PoolHealth, mk_pool_heath },
      pool_assets::{ PoolAssets, prototype, incept },
      tvl::{ TVL, mk_tvl }
   },
   quotes::Quotes
}

pub fn compute_pool_health(q: &Quotes, comp: &Composition,
                           open_pivots: &[Pivot], assets: &[PoolAssets],
                           debug: bool) -> Option<PoolHealth> {
   debug!("compute_pool_health", debug);
   incept(assets).and_then(| incept | {

fn compute_tvl(q: &Quotes, comp: &Composition, open_pivots: &[Pivot],
                   

use book::err_utils::ErrStr;
use crate::types::{ gains::gains, measurable::Measurable, quotes::Quotes };
use super::{ pool_assets::PoolAssets, pool_names::PoolName };

pub fn mk_pool_health(q: &Quotes, name: &PoolName, a: &PoolAssets)
      -> ErrStr<PoolHealth> {

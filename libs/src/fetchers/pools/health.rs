use book::err_utils::ErrStr;
use crate::types::quotes::Quotes;
use super::{ assets::Assets, pool_names::PoolName };

pub fn mk_pool_health(q: &Quotes, name: &PoolName, a: &Assets)
      -> ErrStr<PoolHealth> {

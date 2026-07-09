use chrono::NaiveDate;

use book::{ not_implemented, err_utils::ErrStr };

use crate::{
   fetchers::{ quotes::fetch_quotes, pool_names::fetch_pool_names },
   types::tokens::allocations::pools::Pools
};

pub async fn process_pool_assets(root_url: &str, dt: &NaiveDate)
      -> ErrStr<Pools> {
   let pool_names = fetch_pool_names(root_url).await?;
   let quotes = fetch_quotes(dt).await?;
/*
   let pools: Vec<PoolAssets> =
      async_filter_map(process_each_pool_assets::<Future>(root_url, &quotes), pool_names).await?;
*/
   not_implemented!("process_pool_assets", pool_names, quotes)
}

/*
fn process_each_pool_assets<F>(root_url: &str, q: &Quotes)
      -> impl Fn(Pool) -> F where F: Future<Output = ErrStr<PoolAssets>> {
   move |p: Pool| async {
      let (assets, opens) =
         fetch_assets_and_open_pivots(root_url, q, &p).await?;
   }
   not_implemented!("process_each_pool_assets", root_url, q)
}
*/

// ----- TESTS -------------------------------------------------------

/*
#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
*/

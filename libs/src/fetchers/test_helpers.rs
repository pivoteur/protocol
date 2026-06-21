
#[cfg(not(tarpaulin_include))]
pub mod test_functions {
   use chrono::NaiveDate;
   use book::{ err_utils::ErrStr, tuple_utils::Partition, utils::get_env };
   use crate::{
      fetchers::pivots::fetch_pivots,
      types::{
         aliases::{ Aliases, aliases },
         pivots::Pivot,
         pools::pool_from_str
      }
   };

   pub fn marshall() -> ErrStr<(String, Aliases)> {
      let root_url = get_env("PIVOT_URL")?;
      let a = aliases();
      Ok((root_url, a))
   }

   pub async fn btc_eth_pivots() -> ErrStr<(Partition<Pivot>, NaiveDate)> {
      let (root_url, a) = marshall()?;
      let pool = pool_from_str("btc-eth")?;
      fetch_pivots(&root_url, &pool, &a).await
   }
}

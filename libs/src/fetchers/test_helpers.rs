
#[cfg(not(tarpaulin_include))]
pub mod test_functions {
   use chrono::NaiveDate;
   use book::{
      err_utils::ErrStr,
      file_utils::{ lines_from_file, read_file },
      tuple_utils::Partition,
      utils::get_env
   };
   use crate::{
      fetchers::pivots::{fetch_pivots,parse_pivots},
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
      fetch_pivots(&root_url, &pool, &a, true).await
   }

   pub fn parse_test_pivots_from_file(pool: &str, file_name: &str)
         -> ErrStr<Partition<Pivot>> {
      let pool = pool_from_str(pool)?;
      let pool_data = lines_from_file(file_name)?;
      let a = aliases();
      let (pools, _dt) = parse_pivots(&pool, pool_data, &a, true)?;
      Ok(pools)
   }

   pub fn fetch_local_data(prefix: &str, file: &str) -> ErrStr<String> {
      read_file(&format!("{prefix}/data/{file}"))
   }
}

use chrono::NaiveDate;

use book::{
   err_utils::{ErrStr,not_implemented},
   utils::get_env
};

use libs::{
   fetchers::{fetch_quotes,fetch_assets,fetch_pivots},
   paths::pivot_pool_from_file,
   types::pivots::Pivot
};

const BTC_ETH_PATH: &'static str =
   "pivoteur.github.io/data/pivots/opens/raw/btc-eth.tsv";

fn version() -> String { "2.00".to_string() }
fn app_name() -> String { "aurora".to_string() }
fn usage() -> ErrStr<()> {
   let app = app_name();
   println!("\n{}, version {}\n\n\t$ ./{} <protocol> <date> <path>

Creates virtual pivots based upon available assets.

where
* <protocol> is the protocol, e.g. PIVOT
* <date> to check availability, e.g.: $LE_DATE
* <path> path to the open pivot pool table,
          e.g.: {BTC_ETH_PATH}
", app,  version(), app);
   Err("Needs arguments <protocol> <date>".to_string())
}

// working per pivot pool, first get the assets
async fn new_opens(auth: &str, date: NaiveDate, path: &str)
      -> ErrStr<Vec<Pivot>> {
   let aut = auth.to_uppercase();
   let root_url = get_env(&format!("{aut}_URL"))?;
   let quotes = fetch_quotes(&date).await?;
   let aliases = &quotes.aliases;
   let (prim, piv) = pivot_pool_from_file(path)?;
   let pool_assets = fetch_assets(&root_url, &prim, &piv, aliases).await?;
   let (opens, closes) = fetch_pivots(&root_url, &prim, &piv, aliases).await?;
   not_implemented("new_opens")
}

// ----- TESTS -------------------------------------------------------

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use book::{date_utils::{yesterday,parse_date}, utils::get_args};

   pub async fn runoff_with_args() -> ErrStr<()> {
      let args = get_args();
      if let [auth, dt, path] = args.as_slice() {
         let date = parse_date(&dt)?;
         let _pivs = new_opens(&auth, date, &path).await?;
         Ok(())
      } else {
         usage()
      }
   }

   pub async fn runoff() -> ErrStr<usize> {
      let yday = yesterday();
      let _pivs = new_opens("pivot", yday, BTC_ETH_PATH).await?;
      Ok(1)
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use book::date_utils::yesterday;

   #[tokio::test]
   async fn test_new_opens_ok() {
      let yday = yesterday();
      let pivs = new_opens("pivot", yday, BTC_ETH_PATH).await;
      assert!(pivs.is_ok());
   }
}


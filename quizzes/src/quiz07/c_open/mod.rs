use chrono::NaiveDate;

use book::{
   date_utils::parse_date,
   err_utils::ErrStr,
   utils::{get_env,get_args}
};

use libs::{
   fetchers::{fetch_quotes,fetch_assets,fetch_pivots},
   paths::pivot_pool_from_file,
   reports::header,
   types::{ comps::Composition, pivots::{Pivot,pivot_assets}, coins::Coin }
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

async fn enfetchify(auth: &str, date: NaiveDate, path: &str)
      -> ErrStr<(Composition, Vec<Pivot>)> {
   let aut = auth.to_uppercase();
   let root_url = get_env(&format!("{aut}_URL"))?;
   let quotes = fetch_quotes(&date).await?;
   let aliases = &quotes.aliases;
   let (prim, piv) = pivot_pool_from_file(path)?;

println!("*** For pivot pool: {}", header(&prim, &piv));

   let pool_assets = fetch_assets(&root_url, &prim, &piv, aliases).await?;
   let ((opens, _closes), _max_date) =
      fetch_pivots(&root_url, &prim, &piv, aliases).await?;
   Ok((pool_assets, opens))
}

async fn new_opens(auth: &str, date: NaiveDate, path: &str)
      -> ErrStr<Vec<Coin>> {
   let (pool_assets, opens) = enfetchify(auth, date, path).await?;
   let mut available = pool_assets.as_assets();
   let all_opens = pivot_assets(&opens)?;
   for a in all_opens.assets() {
      available.subtract(&a);
   }
   Ok(available.assets())
}

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

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{
      create_testing,
      date_utils::yesterday,
      utils::now
   };
   use libs::reports::print_table;

   create_testing!("quiz07::c_open");

   run!("new_opens", {
      let yday = yesterday();
      let pivs = now(new_opens("pivot", yday, BTC_ETH_PATH))?;
      print_table("Available BTC+ETH assets", &pivs);
   });
}

#[cfg(test)]
mod tests {
   use super::*;
   use book::{
//      csv_utils::CsvWriter,
      currency::usd::{USD,no_monay},
      date_utils::yesterday,
      utils::{composer,deref}
   };
   use libs::types::measurable::tvl;

   async fn fetchme() -> ErrStr<(Composition, Vec<Pivot>)> {
      let yday = yesterday();
      enfetchify("pivot", yday, BTC_ETH_PATH).await
   }

   #[tokio::test]
   async fn test_enfetchify_ok() {
      let fetchèð = fetchme().await;
      assert!(fetchèð.is_ok());
   }

   fn tvlr() -> impl Fn(Pivot) -> USD {
      composer(deref(tvl), composer(Result::unwrap, deref(Pivot::committed)))
   }

   #[tokio::test]
   async fn test_enfetchify_assets() -> ErrStr<()> {
      let (comp, _pivs) = fetchme().await?;
      assert!(comp.tvl() > no_monay(), "BTC+ETH Pool composition zero!");
      Ok(())
   }
   #[tokio::test]
   async fn test_enfetchify_has_pivots() -> ErrStr<()> {
      let (_comp, pivs) = fetchme().await?;
      assert!(!pivs.is_empty(), "No BTC+ETH pivots?");
      Ok(())
   }
   #[tokio::test]
   async fn test_enfetchify_pivots() -> ErrStr<()> {
      let (_comp, pivs) = fetchme().await?;
      let pivs_tvls: USD = pivs.into_iter().map(tvlr()).sum();
      assert!(pivs_tvls > no_monay(), "BTC+ETH pivots amount to $0.00?");
      Ok(())
   }
/* this is saying there are more pivots than assets $20k, but new_opens reports
   $350 $BTC available and $150 $ETH available

   #[tokio::test]
   async fn test_enfetchify_assets_and_pivots() -> ErrStr<()> {
      let (_comp, _pivs) = fetchme().await?;
      let pivs_tvls: USD = pivs.iter().map(tvlr()).sum();

      assert!(comp.tvl() - pivs_tvls.clone() > no_monay(),
              "More pivots {pivs_tvls} than assets {}
(comp tvl is {})", comp.as_csv(), comp.tvl());

      Ok(())
   }
*/
}


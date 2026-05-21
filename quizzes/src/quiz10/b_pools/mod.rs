use chrono::NaiveDate;

use libs::fetchers::pool_names::fetch_pool_names;

use book::{
   date_utils::parse_date,
   err_utils::ErrStr,
   utils::{ get_args, get_env }
};

fn app_name() -> String { "pools".to_string() }
fn version() -> String { "1.00".to_string() }

async fn print_pool_assets(auth: &str, dt: &NaiveDate) -> ErrStr<()> {
   let ogori_cap = auth.to_uppercase();
   let root_url = get_env(&format!("{ogori_cap}_URL"))?;
   let pools = fetch_pool_names(&root_url).await?;  // TODO: FIXME
   let assets: Vec<String> =
      pools.into_iter().map(|(a,b)| format!("[ '{a}', '{b}' ]")).collect();
   println!("
const poolAssets = {{
   generated: '{dt}',
   assets: [
      {}
   ]
}};
",  assets.join(",\n      "));
   Ok(())
}

fn usage() -> ErrStr<()> {
   println!("
Usage:

$ {} <auth> <date>

Given <auth> access and <date>, {} prints a Javascript object of pool assets.
", app_name(), app_name());
   Err("Need <auth> and <date> arguments".to_string())
}

#[cfg(not(tarpaulin_include))]
pub async fn runoff_get_args() -> ErrStr<()> {
   println!("\n// created by: {}, version: {}\n", app_name(), version());
   let args = get_args();
   if let [auth, dt] = args.as_slice() {
      let date = parse_date(&dt)?;
      print_pool_assets(&auth, &date).await
   } else { usage() }
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;

   use book::{ create_testing, date_utils::today, utils::now };

   create_testing!("quiz10::b_pools");

   run!("pivot_pool_assets", {
      let td = today();
      let auth = "PIVOT";
      let _ = now(print_pool_assets(&auth, &td))?;
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {

   use super::*;
   use book::date_utils::today;

   #[tokio::test]
   async fn test_print_pool_assets_ok() -> ErrStr<()> {
      let td = today();
      let auth = "PIVOT";
      let ans = print_pool_assets(&auth, &td).await;
      assert!(ans.is_ok());
      Ok(())
   }

   #[tokio::test]
   async fn fail_print_pool_assets_bad_auth() {
      let td = today();
      let ans = print_pool_assets("ARBITRAM", &td).await; // geddit?
      assert!(ans.is_err());
   }
}


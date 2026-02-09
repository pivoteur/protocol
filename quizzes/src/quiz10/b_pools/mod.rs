use chrono::NaiveDate;

use libs::git::fetch_pool_names;

use book::err_utils::ErrStr;

fn app_name() -> String { "pools".to_string() }
fn version() -> String { "1.00".to_string() }

async fn print_pool_assets(auth: &str, dt: &NaiveDate) -> ErrStr<()> {
   let ogori_cap = auth.to_uppercase();
   let pools = fetch_pool_names(&ogori_cap, "data/pivots/open/raw").await?;
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

pub mod functional_tests {
   use super::*;

   use book::{
      date_utils::{parse_date,today},
      utils::get_args // ,get_env}
   };

   pub async fn runoff_get_args() -> ErrStr<()> {
      println!("\n// created by: {}, version: {}\n", app_name(), version());
      let args = get_args();
      if let [auth, dt] = args.as_slice() {
         let date = parse_date(&dt)?;
         print_pool_assets(&auth, &date).await
      } else { usage() }
   }

   pub async fn runoff() -> ErrStr<usize> {
      println!("quiz10: b_pools functional test\n");
      let td = today();
      let auth = "PIVOT";
      let _ = print_pool_assets(&auth, &td).await?;
      Ok(1)
   }
}

#[cfg(test)]
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


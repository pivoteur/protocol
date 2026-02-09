use chrono::NaiveDate;

use libs::git::fetch_pool_names;

use book::err_utils::ErrStr;

fn app_name() -> String { "pools".to_string() }
fn version() -> String { "1.00".to_string() }

async fn print_pool_assets(auth: &str, dt: &NaiveDate) -> ErrStr<()> {
   let pools = fetch_pool_names(auth, "data/pivots/open/raw").await?;
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
", app_name(), app_name()):
   Err("Need <auth> and <date> arguments".to_string())
}

pub mod functional_tests {
   use super::*;

   use book::{
      date_utils::{parse_date,today},
      utils::get_args
   };

   pub fn runoff_get_args() -> ErrStr<()> {
      println!("\n// created by: {}, version: {}\n", app_name(), version());
      let args = get_args();
      if let [auth, dt] = args.as_slice() {
         let date = parse_date(&dt)?;
         let _ = print_pool_assets(&auth, &date).await?;
         Ok(())
      } else { usage() }
   }

   pub fn runoff() -> ErrStr<usize> {
      println!("quiz10: b_pools functional test\n");
      let td = today();
      pa

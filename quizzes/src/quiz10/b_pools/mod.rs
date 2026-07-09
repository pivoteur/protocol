use chrono::NaiveDate;
use clap::{ Parser, CommandFactory };

use libs::fetchers::pool_names::fetch_pool_names;

use book::{
   parse_args_add_banner,
   cli_utils::add_banner,
   err_utils::ErrStr,
   string_utils::UppercaseString,
   utils::get_env
};

async fn print_pool_assets(auth: &str, dt: &NaiveDate) -> ErrStr<()> {
   let ogori_cap = auth.to_uppercase();
   let root_url = get_env(&format!("{ogori_cap}_URL"))?;
   let pools = fetch_pool_names(&root_url).await?;  // TODO: FIXME
   let assets: Vec<String> =
      pools.into_iter().map(|pool| {
         let (a, b) = pool.as_tuple();
         format!("[ '{a}', '{b}' ]")
   }).collect();
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

/// pools prints a Javascript object of pool assets.
#[derive(Debug, Parser)]
struct Args {
   /// protocol to determine active pivot pools, e.g.: PIVOT
   protocol: UppercaseString,

   /// date to determine active pivot pools, e.g.: $LE_DATE
   date: NaiveDate
}

#[cfg(not(tarpaulin_include))]
pub async fn runoff_get_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   let vers = Args::command().render_version();
   println!("\n// created by: {vers}\n");
   print_pool_assets(&args.protocol, &args.date).await
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;

   use book::{ create_testing, date_utils::yesterday, utils::now };

   create_testing!("quiz10::b_pools");

   run!("pivot_pool_assets", {
      let _ = now(print_pool_assets("pivot", &yesterday()))?;
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {

   use super::*;
   use book::date_utils::yesterday;

   #[tokio::test]
   async fn test_print_pool_assets_ok() -> ErrStr<()> {
      let ans = print_pool_assets("pivot", &yesterday()).await;
      assert!(ans.is_ok());
      Ok(())
   }

   #[tokio::test]
   async fn fail_print_pool_assets_bad_auth() {
      let ans = print_pool_assets("ARBITRAM", &yesterday()).await; // geddit?
      assert!(ans.is_err());
   }
}


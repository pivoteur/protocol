use chrono::NaiveDate;
use clap::Parser;

use book::{
   parse_args_add_banner,
   cli_utils::add_banner,
   err_utils::ErrStr,
   num::floats::safe_floats::mk_safe_float,
   string_utils::UppercaseString,
   utils::get_env
};

use libs::{
   collections::assets::Assets,
   fetchers::{
      assets::pool::{available_assets_fetcher,subtractor},
      pool_names::fetch_pool_names,
      quotes::fetch_quotes
   },
   types::{ tokens::coins::Coin, comps::Composition }
};

async fn health_computer(f: impl Fn(&mut Assets, &Coin),
                         root_url: &str, date: &NaiveDate, debug: bool) 
      -> ErrStr<Vec<Composition>> {
   if debug { println!("Computing pivot pool health\n"); }
   let pools = fetch_pool_names(&root_url).await?;
   let quotes = fetch_quotes(date).await?;
   let mut ans = Vec::new();
   for pool in pools {
      if debug { println!("Computing health for pool {pool}..."); }
      let comp =
         available_assets_fetcher(&f, &root_url, &quotes, &pool, debug).await?;
      if debug { println!("...done."); }
      ans.push(comp);
   }
   ans.sort_by_key(|c| mk_safe_float(&c.tvl().amount()));
   Ok(ans)
}

async fn compute_health(root_url: &str, date: &NaiveDate, debug: bool)
      -> ErrStr<Vec<Composition>> {
   health_computer(subtractor, root_url, date, debug).await
}

fn composition_as_js_health_row(c: &Composition) -> String {
   format!("{{ pool: {:?}, available: '{}' }}",
           c.pool_name(), c.tvl())
}

fn report_health(dt: NaiveDate, v: Vec<Composition>) -> ErrStr<()> {
   let pools: Vec<String> =
      v.iter().map(composition_as_js_health_row).collect();
   println!("const poolHealth = {{");
   println!("   generated: '{dt}',
   pools = [
      {}
   ]
}};", pools.join("\n      "));
   Ok(())
}

/// prints the current available assets for all pivot pools: a health-check.
#[derive(Debug, Parser)]
#[command(name = "hwaet")]
#[command(version = "1.04")]
struct Args {
   /// protocol to run the health-check on, e.g.: PIVOT
   protocol: UppercaseString,

   /// date on which the health-check data is checked, e.g.: $LE_DATE
   date: NaiveDate,

   /// print debugging-information
   #[arg(short, long)]
   debug: bool
}

pub async fn runoff_with_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   let root_url = get_env(&format!("{}_URL", args.protocol))?;
   let comps = compute_health(&root_url, &args.date, args.debug).await?;
   report_health(args.date, comps)
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod test_functions {
   use super::*;
   pub fn mock_subtractor(_a: &mut Assets, _c: &Coin) { }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use super::test_functions::mock_subtractor as s;
   use paste::paste;
   use book::{ create_testing, date_utils::yesterday, utils::now };
   use libs::fetchers::test_helpers::test_functions::marshall;

   create_testing!("quiz07::d_health");

   run!("compute_health", " (using mock subtractor)", {
      let yday = yesterday();
      let (root_url, _) = marshall()?;
      let comps = now(health_computer(s, &root_url, &yday, true))?;
      report_health(yday, comps)?;
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use super::test_functions::mock_subtractor as s;
   use std::collections::HashSet;
   use book::date_utils::yesterday;
   use libs::{
      fetchers::test_helpers::test_functions::marshall,
      types::pools::Pool
   };

   #[tokio::test]
   async fn test_compute_health_ok_mock_subtractor() -> ErrStr<()> {
      let (url, _) = marshall()?;
      assert!(health_computer(s, &url, &yesterday(), false).await.is_ok());
      Ok(())
   }

   #[tokio::test]
   async fn test_compute_health_all_pools_with_debug_mock_subtractor()
         -> ErrStr<()> {
      let (root_url, _) = marshall()?;
      let yday = yesterday();
      let npools = fetch_pool_names(&root_url).await?;
      let pool_names: HashSet<String> =
         npools.iter().map(Pool::pool_name).collect();
      let assets = health_computer(s, &root_url, &yday, true).await?;
      let al = &assets.len();
      let pl = &pool_names.len();
      assert_eq!(pl, al, "Assets {al} do not equal pools {pl}!");
      for a in assets {
         let asset = a.pool_name();
         assert!(pool_names.contains(&asset),
                 "I do not know this pool: {asset}");
      }
      Ok(())
   }
}


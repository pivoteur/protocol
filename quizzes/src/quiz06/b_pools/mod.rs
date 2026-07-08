use clap::Parser;

use book::{
   parse_args_add_banner,
   cli_utils::add_banner,
   currency::usd::{USD,mk_usd,no_monay},
   csv_utils::CsvWriter,
   err_utils::ErrStr,
   string_utils::{ UppercaseString, s },
   tuple_utils::Partition,
   utils::get_env
};

use libs::{
   fetchers::{ pool_names::fetch_pool_names, assets::pool::fetch_assets },
   reports::{print_table,total_line},
   types::{
      aliases::aliases,
      comps::{Composition,total,last_updated},
      measurable::sort_by_size
  }
};

fn report_on_assets(pools: Vec<Composition>, min_val: USD) -> ErrStr<()> {
   let skip = if let Some(a_pool) = pools.first() {
      Ok(a_pool.ncols())
   } else {
      Err(s("Portfolio has no pivot pools!"))
   }? - 3;
   let (mut viab, mut poor): Partition<_>
      = pools.into_iter().partition(|p| p.tvl() > min_val);
   let sz1 = print_update(skip, "main pools", &mut viab);
   let sz2 = print_update(skip, "pools to review", &mut poor);
   total_line(skip, " ,total", &(sz1 + sz2));
   Ok(())
}

fn print_update(skip: usize, title: &str, pools: &mut Vec<Composition>) -> USD {
   if let Some(updated) = last_updated(pools) {
      pools.sort_by(sort_by_size);
      let tot = total(pools);
      let header = format!("{title},updated:,{updated}");
      print_table(&header, pools);
      total_line(skip, &format!("{title},subtotal"), &tot);
      tot
   } else {
      println!("\nNo {title}");
      no_monay()
   }
}

/// Reports pivot pools' TVL (total value locked)
#[derive(Debug, Parser)]
struct Args {
   /// protocol of pools processed
   protocol: UppercaseString,

   /// minimum pivot pool NAV filter
   #[arg(short, long, default_value_t = 10000.0)]
   min: f32,

   /// print debugging information
   #[arg(short, long)]
   debug: bool
}

pub async fn runoff_get_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   process_pool_assets(&args.protocol, args.min, args.debug).await
}

async fn process_pool_assets(protocol: &str, min: f32, debug: bool)
      -> ErrStr<()> {
   let root_url = get_env(&format!("{protocol}_URL"))?;
   let pools = fetch_all_pools_assets(&root_url, debug).await?;
   report_on_assets(pools, mk_usd(min))
}

async fn fetch_all_pools_assets(root_url: &str, debug: bool)
      -> ErrStr<Vec<Composition>> {
   let aliases = aliases();
   let pool_names = fetch_pool_names(&root_url).await?;
   let mut pools = Vec::new();
   for pool in pool_names {
      let p = fetch_assets(&root_url, &pool, &aliases, debug).await?;
      pools.push(p);
   }
   Ok(pools)
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {

   use super::*;
   use paste::paste;
   use book::{ create_testing, utils::now };

   create_testing!("quiz06::b_pools");

   run!("fetch_all_pool_assets", {
      let _ = now(process_pool_assets("PIVOT", 10000.0, true));
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {

   use super::*;
   use book::utils::get_env;

   async fn fetch_pools() -> ErrStr<Vec<Composition>> {
      let root_url = get_env("PIVOT_URL")?;
      fetch_all_pools_assets(&root_url, true).await
   }

   #[tokio::test] async fn test_fetch_all_pools_assets_ok() -> ErrStr<()> {
      let pools = fetch_pools().await;
      assert!(pools.is_ok());
      Ok(())
   }

   #[tokio::test] async fn test_fetch_all_pools_have_assets() -> ErrStr<()> {
      let pools = fetch_pools().await?;
      assert!(!pools.is_empty());
      Ok(())
   }
}

use book::{
   currency::usd::{USD,mk_usd,no_monay},
   csv_utils::CsvWriter,
   err_utils::ErrStr,
   num_utils::parse_or
};

use libs::{
   fetchers::fetch_assets,
   git::fetch_pool_names,
   reports::{print_table,total_line},
   types::{
      comps::{Composition,total,last_updated},
      measurable::sort_by_size
  }
};

fn version() -> String { "1.06".to_string() }
fn app_name() -> String { "assets".to_string() }
fn min_default() -> f32 { 10000.0 }
fn min_value(mini: Option<&String>) -> USD {
   mk_usd(parse_or(mini, min_default()))
}

fn report_on_assets(pools: Vec<Composition>, min_val: USD) {
   let skip = if let Some(a_pool) = pools.first() { a_pool.ncols() } else {
      panic!("Portfolio has no pivot pools!")
   } - 3;
   let (mut viab, mut poor): (Vec<_>, Vec<_>)
      = pools.into_iter().partition(|p| p.tvl() > min_val);
   let sz1 = print_update(skip, "main pools", &mut viab);
   let sz2 = print_update(skip, "pools to review", &mut poor);
   total_line(skip, " ,total", &(sz1 + sz2));
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

fn usage() -> String {
   let dapp = app_name();
   let minime = min_default();
   println!("Reports pivot pools' TVL (total value locked)

Usage:

$ {dapp} <protocol> [min={minime}]

where

* <protocol> is the dapp processing the pools
* [min] minimum pool TVL, default {minime}
");
   "<protocol>-argument missing.".to_string()
}

async fn fetch_all_pools_assets(auth: &str, root_url: &str)
      -> ErrStr<Vec<Composition>> {
   let pool_names = fetch_pool_names(&auth, "data/pools").await?;
   let mut pools = Vec::new();
   for (prim, piv) in pool_names {
      let pool = fetch_assets(&root_url, &prim, &piv).await?;
      pools.push(pool);
   }
   Ok(pools)
}

pub mod functional_tests {

   use super::*;

   use book::utils::{get_args,get_env};

   pub async fn runoff_get_args() -> ErrStr<()> {
      let args = get_args();
      do_it(args.first(), args.last()).await
   }

   async fn do_it(mb_auth: Option<&String>, mb_mini: Option<&String>)
         -> ErrStr<()> {
      println!("{}, version: {}\n", app_name(), version());
      let auth = mb_auth.ok_or_else(|| usage())?.to_uppercase();
      let root_url = get_env(&format!("{auth}_URL"))?;
      let mini = min_value(mb_mini);
      let pools = fetch_all_pools_assets(&auth, &root_url).await?;
      report_on_assets(pools, mini);
      Ok(())
   }

   pub async fn runoff() -> ErrStr<usize> {
      let pools =
         do_it(Some(&"PIVOT".to_string()), Some(&"10000".to_string())).await;
      match pools {
         Ok(_) => Ok(1),
         Err(x) => Err(x)
      }
   }
}

#[cfg(test)]
mod tests {

   use super::*;

   use book::utils::get_env;

   async fn fetch_pools() -> ErrStr<Vec<Composition>> {
      let root_url = get_env("PIVOT_URL")?;
      fetch_all_pools_assets("PIVOT", &root_url).await
   }

   #[tokio::test]
   async fn test_fetch_all_pools_assets_ok() -> ErrStr<()> {
      let pools = fetch_pools().await;
      assert!(pools.is_ok());
      Ok(())
   }

   #[tokio::test]
   async fn test_fetch_all_pools_have_assets() -> ErrStr<()> {
      let pools = fetch_pools().await?;
      assert!(!pools.is_empty());
      Ok(())
   }

   #[test]
   fn test_min_default_none() {
      assert_eq!(mk_usd(min_default()), min_value(None));
   }

   #[test]
   fn test_min_default_parse_failed() {
      assert_eq!(mk_usd(min_default()),
                 min_value(Some(&"blad-di-blah".to_string())));
   }

   #[test]
   fn test_min_value() {
      assert_eq!(mk_usd(1234.0), min_value(Some(&"1234".to_string())));
   }
}


use book::{
   currency::usd::mk_usd,
   err_utils::ErrStr,
   num_utils::parse_or,
   utils::{get_args,get_env}
};

use libs::{
   fetchers::fetch_assets,
   git::fetch_pool_names,
   // types::comps::Composition,
   reports::print_table,
   types::measurable::sort_by_size
};

fn version() -> String { "1.01".to_string() }
fn app_name() -> String { "assets".to_string() }
fn min_default() -> f32 { 10000.0 }

#[tokio::main]
async fn main() -> ErrStr<()> {
   let args = get_args();
   let auth = args.first().ok_or_else(|| usage())?.to_uppercase();
   let root_url = get_env(&format!("{auth}_URL"))?;
   let min_val = mk_usd(parse_or(args.last(), min_default()));
   let pool_names = fetch_pool_names(&auth, "data/pools").await?;
   let mut pools = Vec::new();
   for (prim, piv) in pool_names {
      let pool = fetch_assets(&root_url, &prim, &piv).await?;
      pools.push(pool);
   }
   let (mut viab, mut poor): (Vec<_>, Vec<_>)
      = pools.into_iter().partition(|p| p.tvl() > min_val);
   viab.sort_by(sort_by_size);
   poor.sort_by(sort_by_size);
   println!("{}, version: {}", app_name(), version());
   print_table("Power Pools", &viab);
   print_table("Pools to review", &poor);
   Ok(())
}

fn usage() -> String {
   let dapp = app_name();
   let minime = min_default();
   println!("{dapp}

Reports pivot pools' TVL (total value locked)

Usage:

$ {dapp} <protocol> [min={minime}]

where

* <protocol> is the dapp processing the pools
* [min] minimum pool TVL, default {minime}
");
   "<protocol>-argument missing.".to_string()
}


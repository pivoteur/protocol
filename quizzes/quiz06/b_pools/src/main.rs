use book::{
   currency::usd::{USD,mk_usd,no_monay},
   csv_utils::{CsvWriter,mk_blank},
   err_utils::ErrStr,
   num_utils::parse_or,
   utils::{get_args,get_env}
};

use libs::{
   fetchers::fetch_assets,
   git::fetch_pool_names,
   reports::{print_table,header},
   types::{
      comps::{Composition,total,last_updated},
      measurable::sort_by_size
  }
};

fn version() -> String { "1.04".to_string() }
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
      eprintln!("*** processing pool {}", header(&prim, &piv));
      let pool = fetch_assets(&root_url, &prim, &piv).await?;
      pools.push(pool);
   }
   report_on_assets(pools, min_val);
   Ok(())
}

fn report_on_assets(pools: Vec<Composition>, min_val: USD) {
   let skip = if let Some(a_pool) = pools.first() { a_pool.ncols() } else {
      panic!("Portfolio has no pivot pools!")
   } - 3;
   let (mut viab, mut poor): (Vec<_>, Vec<_>)
      = pools.into_iter().partition(|p| p.tvl() > min_val);
   println!("{}, version: {}", app_name(), version());
   let sz1 = print_update(skip, "main pools", &mut viab);
   let sz2 = print_update(skip, "pools to review", &mut poor);
   footer(skip, " ,total", &(sz1 + sz2));
}

fn print_update(skip: usize, title: &str, pools: &mut Vec<Composition>) -> USD {
   if let Some(updated) = last_updated(pools) {
      pools.sort_by(sort_by_size);
      let tot = total(pools);
      let header = format!("{title},updated:,{updated}");
      print_table(&header, pools);
      footer(skip, &format!("{title},subtotal"), &tot);
      tot
   } else {
      println!("\nNo {title}");
      no_monay()
   }
}

fn footer(skip: usize, header: &str, total: &USD) {
   let pre = mk_blank(skip);
   println!("\n{}{header}:,{total}", pre.as_csv());
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


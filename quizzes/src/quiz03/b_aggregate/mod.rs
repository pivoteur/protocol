use chrono::NaiveDate;
use book::{
   date_utils::parse_date,
   err_utils::ErrStr,
   utils::get_args
};
use libs::{
   reports::print_table,
   fetchers::{ pivots::fetch_pivots, quotes::fetch_quotes },
   types::{ 
      pivots::{ partition_on, next_close_id },
      pools::{Pool,mk_pool},
      proposals::proposes::propose
   }
};

fn app_name() -> String { "basset".to_string() }
fn version() -> String { "1.00".to_string() }

fn usage() -> ErrStr<()> {
   println!("Usage:
   $ {} <root URL> <primary asset> <pivot asset> <date>

      Partitions open pivots then aggregates proposed close pivots.
      The pivot pools are reposed (in git, currently) at <root URL>.
      Open pivots are stored as raw-CSV files in git at protocol <root URL>.",
       app_name());
   Err("Needs <root URL> <primary> <pivot> <date> arguments".to_string())
}

async fn aggregate(root_url: &str, pool: &Pool, date: NaiveDate)
      -> ErrStr<()> {
   let quotes = fetch_quotes(&date).await?;
   let a = &quotes.aliases;
   let ((opns, cls), max_date) = fetch_pivots(root_url, pool, a).await?;
   let next_close = next_close_id(&cls);
   preamble(pool, opns.len(), &max_date, &date);
   let proposer = propose(&quotes);
   let (prim, _piv) = pool.as_tuple();
   let (lefts, rights) = partition_on(&prim, opns);
   let mut props = Vec::new();
   let follow = if let Some((prop, nxt)) = proposer((lefts, next_close))? {
      props.push(prop);
      nxt
   } else {
      next_close
   };
   if let Some((prop, _)) = proposer((rights, follow))? {
      props.push(prop);
   }
   print_table("No close pivots", &props);
   Ok(())
}

fn preamble(pool: &Pool, len: usize, max_date: &NaiveDate, date: &NaiveDate) {
   let header = pool.pool_name();
   let pool = format!("{header} pivot pool");

   println!("{header}\n");
   println!("There are {len} open pivots for the {pool}.");
   println!("The last entry is on {max_date}.");
   println!("Recommendations are made for token quotes on {date}.\n");
}

pub async fn runoff_get_args() -> ErrStr<()> {
   println!("{}, version: {}", app_name(), version());
   if let [root_url, prim, piv, date] = get_args().as_slice() {
      let dt = parse_date(&date)?;
      let pool = mk_pool(&prim, &piv);
      aggregate(root_url, &pool, dt).await
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
   use book::{ create_testing, utils::get_env };

   create_testing!("quiz03::b_aggregate");

   run!("aggregate", {
      println!("\nquiz03: b_aggregate functional test\n");
      let piv = get_env("PIVOT_URL")?;
      let dt = parse_date("2026-02-02")?;
      let pool = mk_pool("btc", "eth");
      let _ = aggregate(&piv, &pool, dt);
   });
}

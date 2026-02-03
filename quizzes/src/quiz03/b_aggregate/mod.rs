use chrono::NaiveDate;

use book::{
   date_utils::parse_date,
   err_utils::ErrStr,
};

use libs::{
   fetchers::{fetch_pivots,fetch_quotes},
   reports::print_table,
   types::pivots::{partition_on,next_close_id,propose}
};

async fn aggregate(root_url: &str, prim: &str, piv: &str, date: NaiveDate)
      -> ErrStr<()> {
   let (opens, closes, max_date) = fetch_pivots(root_url, prim, piv).await?;
   let quotes = fetch_quotes(&date).await?;
   let next_close = next_close_id(&closes);
   preamble(prim, piv, opens.len(), &max_date, &date);
   let proposer = propose(&quotes);

   let (lefts, rights) = partition_on(prim, opens);
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

fn preamble(prim: &str, piv: &str, len: usize,
            max_date: &NaiveDate, date: &NaiveDate) {
   let cap_prim = prim.to_uppercase();
   let cap_piv = piv.to_uppercase();
   let header = format!("{cap_prim}+{cap_piv}");
   let pool = format!("{header} pivot pool");

   println!("{header}\n");
   println!("There are {len} open pivots for the {pool}.");
   println!("The last entry is on {max_date}.");
   println!("Recommendations are made for token quotes on {date}.\n");
}

fn app_name() -> String { "basset".to_string() }
fn version() -> String { "1.00".to_string() }

fn usage() -> ErrStr<()> {
   println!("Usage:

	$ {} <root URL> <primary asset> <pivot asset> <date>

Partitions open pivots then aggregates proposed close pivots.

The pivot pools are reposed (in git, currently) at <root URL>.

Open pivots are stored as raw-CSV files in git at protocol <root URL>.
", app_name());
   Err("Needs <root URL> <primary> <pivot> <date> arguments".to_string())
}

pub mod functional_tests {
   use super::*;

   use book::utils::{get_args,get_env};

   pub async fn runoff() -> ErrStr<usize> {
      println!("\nquiz03: b_aggregate functional test\n");
      let piv = get_env("PIVOT_URL")?;
      let dt = parse_date("2026-02-02")?;
      let _ = aggregate(&piv, "BTC", "ETH", dt).await?;
      Ok(1)
   }

   pub async fn runoff_get_args() -> ErrStr<()> {
      println!("{}, version: {}", app_name(), version());
      if let [root_url, prim, piv, date] = get_args().as_slice() {
         let dt = parse_date(&date)?;
         aggregate(root_url, prim, piv, dt).await
      } else {
         usage()
      }
   }
}


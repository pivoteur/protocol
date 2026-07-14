use chrono::NaiveDate;
use clap::Parser;

use book::{
   parse_args_add_banner,
   cli_utils::add_banner,
   err_utils::ErrStr,
   string_utils::UppercaseString,
   utils::get_env
};

use libs::{
   reports::print_table,
   fetchers::{ pivots::fetch_pivots, quotes::fetch_quotes },
   types::{ 
      pivots::opens::{ partition_on, next_close_id },
      pools::{Pool,mk_pool},
      proposals::proposes::propose
   }
};

async fn aggregate(root_url: &str, pool: &Pool, date: NaiveDate, debug: bool)
      -> ErrStr<()> {
   let quotes = fetch_quotes(&date).await?;
   let a = &quotes.aliases;
   let ((opns, cls), max_date) = fetch_pivots(root_url, pool, a, debug).await?;
   let next_close = next_close_id(&cls);
   preamble(pool, opns.len(), &max_date, &date);
   let proposer = propose(&quotes);
   let (prim, _piv) = pool.as_tuple();
   let (lefts, rights) = partition_on(&prim, opns);
   let mut props = Vec::new();
   let follow =
      if let Some((prop, nxt)) = proposer((lefts, next_close), debug)? {
         props.push(prop);
         nxt
   } else {
         next_close
   };
   if let Some((prop, _)) = proposer((rights, follow), debug)? {
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

/// Partitions open pivots then aggregates proposed close pivots.
///
/// The pivot pools are reposed (in git, currently) at <root URL>.
/// pivots are stored as raw-CSV files in git at protocol <root URL>.",
#[derive(Parser,Debug)]
struct Args {
   /// Protocol holding the assets, e.g.: PIVOT
   protocol: UppercaseString,

   /// Primary asset of pivot pool, e.g.: BTC
   primary: String,

   /// Pivot asset of the pivot pool, e.g.: ETH
   pivot: String,

   /// Date to assess the assets, usually $LE_DATE
   date: NaiveDate,

   /// Show debugging information
   #[arg(short, long)]
   debug: bool
}

pub async fn runoff_get_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   let pool = mk_pool(&args.primary, &args.pivot);
   let root_url = get_env(&format!("{}_URL", args.protocol))?;
   aggregate(&root_url, &pool, args.date, args.debug).await
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, date_utils::yesterday, utils::now };

   create_testing!("quiz03::b_aggregate");

   run!("aggregate", {
      let piv = get_env("PIVOT_URL")?;
      let pool = mk_pool("btc", "eth");
      let _ = now(aggregate(&piv, &pool, yesterday(), true));
   });
}

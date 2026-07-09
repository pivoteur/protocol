use clap::Parser;

use book::{
   parse_args_add_banner,
   cli_utils::add_banner,
   csv_utils::{ CsvHeader, print_csv },
   err_utils::ErrStr,
   string_utils::UppercaseString,
   utils::get_env
};

use libs::{ 
   fetchers::pivots::fetch_pivots,
   types::{
      aliases::aliases,
      pivots::{Pivot,partition_on},
      pools::{Pool,mk_pool}
   }
};

fn list_open_pivots(piv: &str, opens: Vec<Pivot>) {
   if opens.is_empty() {
      println!("No open {piv} pivots\n");
   } else {
      println!("{piv} open pivots:\n");
      let mut print_header = true;
      for o in opens {
         if print_header {
            println!("{}",o.header());
            print_header = false;
         }
         print_csv(&o);
      }
      println!("");
   }
}

#[derive(Debug,Parser)]
/// Partitions open pivots.
///
/// The pivot pools are reposed (in git, currently) at <root URL>.
/// Open pivots are stored as raw-CSV files in git at protocol <root URL>.
struct Args {
   /// Protocol for which to partition pivots, e.g.: PIVOT
   protocol: UppercaseString,

   /// Primary pivot pool asset, e.g.: BTC
   primary: String,

   /// Pivot pivot pool asset, e.g.: ETH
   pivot: String,

   /// Show debugging information
   #[arg(short, long)]
   debug: bool
}

pub async fn runoff_get_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   let pool = mk_pool(&args.primary, &args.pivot);
   let root_url = get_env(&format!("{}_URL", args.protocol))?;
   fetch_and_list_open_pivots(&root_url, &pool, args.debug).await
}

async fn fetch_and_list_open_pivots(root_url: &str, pool: &Pool, debug: bool)
      -> ErrStr<()> {
   let a = aliases();
   let ((opens, _closes), _max_date) =
      fetch_pivots(root_url, pool, &a, debug).await?;
   let (prim, piv) = pool.as_tuple();
   let (lefts, rights) = partition_on(&prim, opens);
   list_open_pivots(&prim, lefts);
   list_open_pivots(&piv, rights);
   Ok(())
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, err_utils::ErrStr, utils:: { get_env, now } };
   use libs::types::pools::pool_from_str;

   create_testing!("quiz03::a_partition");

   run!("partition", {
      let root_url = get_env("PIVOT_URL")?;
      let pool = pool_from_str("btc-eth")?;
      let _ = now(fetch_and_list_open_pivots(&root_url, &pool, true));
   });
}

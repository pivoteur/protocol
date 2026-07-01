use book::{ 
   err_utils::ErrStr,
   csv_utils::{ CsvHeader, print_csv },
   utils::get_args
};
use libs::{ 
   fetchers::pivots::fetch_pivots,
   types::{
      aliases::aliases,
      pivots::{Pivot,partition_on},
      pools::{Pool,mk_pool}
   }
};

fn usage() -> ErrStr<()> {
   println!("Usage:

$ cargo run <root URL> <primary asset> <pivot asset>

Partitions open pivots.

The pivot pools are reposed (in git, currently) at <root URL>.

Open pivots are stored as raw-CSV files in git at protocol <root URL>.
");
   Err("Needs <root URL> <primary> <pivot> arguments".to_string())
}

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

pub async fn runoff_get_args() -> ErrStr<()> {
   if let [root_url, prim, piv] = get_args().as_slice() {
      let pool = mk_pool(&prim, &piv);
      fetch_and_list_open_pivots(root_url, &pool).await
   } else {
      usage()
   }
}

async fn fetch_and_list_open_pivots(root_url: &str, pool: &Pool) -> ErrStr<()> {
   let a = aliases();
   let ((opens, _closes), _max_date) =
      fetch_pivots(root_url, pool, &a).await?;
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

   create_testing!("quiz03::a_partition", "", true);

   run!("partition", {
      let root_url = get_env("PIVOT_URL")?;
      let pool = pool_from_str("btc-eth")?;
      let _ = now(fetch_and_list_open_pivots(&root_url, &pool));
   });
}

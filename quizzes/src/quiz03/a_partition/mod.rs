use book:: { 
   err_utils::ErrStr,
   csv_utils:: { CsvHeader, print_csv },
   utils::get_args
};
use libs:: { 
   fetchers::pivots::fetch_pivots,
   types:: { 
      pivots::Pivot,
      aliases::aliases,
      pivots::partition_on
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
      fetch_and_list_open_pivots(root_url, prim, piv).await
   } else {
      usage()
   }
}

async fn fetch_and_list_open_pivots(root_url: &str, prim: &str, piv: &str) -> ErrStr<()> {
   let a = aliases();
   let ((opens, _closes), _max_date) =
      fetch_pivots(root_url, prim, piv, &a).await?;
   let (lefts, rights) = partition_on(prim, opens);
   list_open_pivots(prim, lefts);
   list_open_pivots(piv, rights);
   Ok(())
}

// ----- TESTS -------------------------------------------------------
#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{
      err_utils::ErrStr,
      utils:: { get_env, now },
      create_testing
   };

   
   create_testing!("quiz03::a_partition");
   
   run!("partition", {
      let root_url = get_env("PIVOT_URL")?;
      match now(fetch_and_list_open_pivots(&root_url, "BTC", "ETH")) {
         Ok(_) => Ok(1),
         Err(x) => Err(x)
      }
   });
   
}

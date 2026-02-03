use book::csv_utils::{CsvHeader,print_csv};

use libs::types::pivots::Pivot;

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

pub mod functional_tests {

   use book::{
      err_utils::ErrStr,
      utils::{get_args,get_env}
   };

   use libs::{
      fetchers::fetch_pivots,
      types::pivots::partition_on
   };

   use super::*;

   pub async fn runoff_get_args() -> ErrStr<()> {
      if let [root_url, prim, piv] = get_args().as_slice() {
         do_it(root_url, prim, piv).await
      } else {
         usage()
      }
   }

   pub async fn run_partition() -> ErrStr<usize> {
      let root_url = get_env("PIVOT_URL")?;
      match do_it(&root_url, "BTC", "ETH").await {
         Ok(_) => Ok(1),
         Err(x) => Err(x)
      }
   }
   
   pub async fn runoff() -> ErrStr<usize> {
      println!("\nquiz03: a_partition\n");
      run_partition().await
   }

   async fn do_it(root_url: &str, prim: &str, piv: &str) -> ErrStr<()> {
      let (opens, _closes, _max_date) =
         fetch_pivots(root_url, prim, piv).await?;
      let (lefts, rights) = partition_on(prim, opens);
      list_open_pivots(prim, lefts);
      list_open_pivots(piv, rights);
      Ok(())
   }

   fn usage() -> ErrStr<()> {
      println!("Usage:

	$ cargo run <root URL> <primary asset> <pivot asset>

Partitions open pivots.

The pivot pools are reposed (in git, currently) at <root URL>.

Open pivots are stored as raw-CSV files in git at protocol <root URL>.
");
       Err("Needs <root URL> <primary> <pivot> arguments".to_string())
   }
}

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

   use chrono::NaiveDate;

   use book::{
      csv_utils::{print_csv,CsvHeader},
      date_utils::parse_date,
      err_utils::ErrStr,
      test_utils::{collate_results,mk_tests},
      utils::{get_args,get_env}
   };

   use libs::{
      fetchers::fetch_pivots,
      types::pivots::{partition_on,Pivot}
   };

   pub async fn runoff_get_args() -> ErrStr<()> {
      if let [root_url, prim, piv, date] = get_args().as_slice() {
         let dt = parse_date(&date)?;
         do_it(root_url, prim, piv, dt).await
      } else {
         usage()
      }
   }

   pub async fn run_partition() -> ErrStr<()> {
      let root_url = get_env("PIVOT_URL")?;
      let dt = parse_date("2026-01-30")?;
      do_it(&root_url, "BTC", "ETH", dt).await
   }
   
   pub async fn runoff() -> ErrStr<()> {
      collate_results("quiz03: a_partition",
         &mk_tests("run_partition", vec![run_partition()]))
   }

   async fn do_it(root_url: &str, prim: &str, piv: &str, _date: NaiveDate)
         -> ErrStr<()> {
      let (opens, _closes, _max_date) =
         fetch_pivots(root_url, prim, piv).await?;
      let (lefts, rights) = partition_on(prim, opens);
      list_open_pivots(prim, lefts);
      list_open_pivots(piv, rights);
      Ok(())
   }

   fn usage() -> ErrStr<()> {
      println!("Usage:

	$ cargo run <root URL> <primary asset> <pivot asset> <date>

Partitions open pivots.

The pivot pools are reposed (in git, currently) at <root URL>.

Open pivots are stored as raw-CSV files in git at protocol <root URL>.
");
       Err("Needs <root URL> <primary> <pivot> <date> arguments".to_string())
    }
}

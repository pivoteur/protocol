use chrono::NaiveDate;

use book::{
   csv_utils::print_csv,
   date_utils::parse_date,
   err_utils::ErrStr,
   utils::get_args
};

use libs::{
   fetchers::fetch_pivots,
   types::pivots::{partition_on,Pivot}
};

#[tokio::main]
async fn main() -> ErrStr<()> {
   if let [root_url, prim, piv, date] = get_args().as_slice() {
      let dt = parse_date(&date)?;
      do_it(root_url, prim, piv, dt).await
   } else {
      usage()
   }
}

async fn do_it(root_url: &str, prim: &str, piv: &str, _date: NaiveDate)
      -> ErrStr<()> {
   let (opens, _closes, _max_date) = fetch_pivots(root_url, prim, piv).await?;
   let (lefts, rights) = partition_on(prim, opens);
   pivs(prim, lefts);
   pivs(piv, rights);
   Ok(())
}

fn pivs(piv: &str, opens: Vec<Pivot>) {
   println!("{piv} open pivots:\n");
   for o in opens {
      print_csv(&o);
   }
   println!("");
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

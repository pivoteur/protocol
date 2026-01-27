use chrono::NaiveDate;

use book::{
   csv_utils::print_csv,
   date_utils::parse_date,
   err_utils::ErrStr,
   utils::get_args
};

use libs::{
   fetchers::{fetch_pivots,fetch_quotes},
   types::{
      pivots::{next_close_id,propose},
      util::CsvHeader
   }
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

async fn do_it(root_url: &str, prim: &str, piv: &str, date: NaiveDate)
      -> ErrStr<()> {
   let (opens, closes, max_date) = fetch_pivots(root_url, prim, piv).await?;
   let quotes = fetch_quotes(&date).await?;
   let mut next_close = next_close_id(&closes);
   let mut print_header: bool = true;
   let proposer = propose(&quotes);
   let cap_prim = prim.to_uppercase();
   let cap_piv = piv.to_uppercase();
   let header = format!("{cap_prim}+{cap_piv}");
   let pool = format!("{header} pivot pool");
   let len = &opens.len();

   println!("{header}\n");
   println!("There are {len} open pivots for the {pool}.");
   println!("The last entry is on {max_date}.");
   println!("Recommendations are made for token quotes on {date}.\n");

   for h in opens {
      if let Some((prop, next_next)) = proposer((h, next_close))? {
         if print_header {
            println!("{}",prop.header());
            print_header = false;
         }
         print_csv(&prop);
         next_close = next_next;
      }
   }

   let no_close_pivots = print_header;
   if no_close_pivots {
      println!("No close pivot recommendations for {pool}.");
   }
   Ok(())
}

fn usage() -> ErrStr<()> {
   println!("Usage:

	$ cargo run <root URL> <primary asset> <pivot asset> <date>

Proposes close pivots for the <prim>+<piv> pivot pool for <date>.
The pivot pools are reposed (in git, currently) at <root URL>.

Open pivots are stored as raw-CSV files in git at protocol <root URL>.
");
   Err("Needs <root URL> <primary> <pivot> <date> arguments".to_string())
}

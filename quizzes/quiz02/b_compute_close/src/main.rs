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
   if let [prim, piv, date] = get_args().as_slice() {
      let dt = parse_date(&date)?;
      do_it(prim, piv, dt).await
   } else {
      usage()
   }
}

async fn do_it(prim: &str, piv: &str, date: NaiveDate) -> ErrStr<()> {
   let (hbars, etc) = fetch_pivots(prim, piv).await?;
   let quotes = fetch_quotes(&date).await?;
   let mut next_close = next_close_id(&etc);
   let mut print_header: bool = true;
   let proposer = propose(&quotes);
   for h in hbars {
      if let Some((prop, next_next)) = proposer((h, next_close))? {
         if print_header {
            println!("{}",prop.header());
            print_header = false;
         }
         print_csv(&prop);
         next_close = next_next;
      }
   }
   
   Ok(())
}

fn usage() -> ErrStr<()> {
   println!("Usage:

	$ cargo run <primary asset> <pivot asset> <date>

Proposes close pivots for the <prim>+<piv> pivot pool for <date>.
");
   Err("Needs <primary> <pivot> <date> arguments".to_string())
}

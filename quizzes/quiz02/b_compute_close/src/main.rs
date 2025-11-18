use chrono::NaiveDate;

use book::{
   csv_utils::print_csv,
   date_utils::parse_date,
   err_utils::ErrStr,
   utils::get_args
};

use libs::{
   fetchers::{fetch_open_pivots,fetch_quotes},
   types::util::CsvHeader
};

#[tokio::main]
async fn main() -> ErrStr<()> {
   if let Some(date) = get_args().first() {
      let dt = parse_date(&date)?;
      do_it(dt).await
   } else {
      usage()
   }
}

async fn do_it(date: NaiveDate) -> ErrStr<()> {
   let hbars = fetch_open_pivots("HBAR", "USDC").await?;
   let quotes = fetch_quotes(&date).await?;
   let mut print_header: bool = true;
   for h in hbars {
      if print_header {
         println!("{}",h.header());
         print_header = false;
      }
      print_csv(&h);
   }
   println!("\nquotes for today: {quotes:?}");
   Ok(())
}

fn usage() -> ErrStr<()> {
   println!("Usage:

	$ cargo run <date>

Shows the open pivots and the token-prices for <date>.
");
   Err("Needs <date> argument".to_string())
}

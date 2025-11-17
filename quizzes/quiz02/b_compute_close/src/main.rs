use book::{
   csv_utils::print_csv,
   err_utils::ErrStr
};

use libs::{
   fetchers::fetch_open_pivots,
   types::util::CsvHeader
};

#[tokio::main]
async fn main() -> ErrStr<()> {
   let hbars = fetch_open_pivots("HBAR", "USDC").await?;
   let mut print_header: bool = true;
   for h in hbars {
      if print_header {
         println!("{}",h.header());
         print_header = false;
      }
      print_csv(&h);
   }
   Ok(())
}


use book::{
   csv_utils::CsvWriter,
   date_utils::parse_date,
   err_utils::ErrStr,
   utils::get_args
};

use libs::{
   fetchers::{assets::pool::fetch_available_assets,quotes::fetch_quotes},
   types::util::{pool_from_str,pool_name}
};

fn version() -> String { "2.01".to_string() }
fn app_name() -> String { "aurora".to_string() }
fn usage() -> ErrStr<()> {
   let app = app_name();
   println!("\n{}, version {}\n\n\t$ {} <protocol> <date> <path>

Creates virtual pivots based upon available assets.

where
* <protocol> is the protocol, e.g. PIVOT
* <date> to check availability, e.g.: $LE_DATE
* <path> path to pivot pool to open pivots,
  e.g.: data/pivots/open/raw/btc-eth.tsv
", app,  version(), app);
   Err("Needs arguments <protocol> <date> <path>".to_string())
}

pub async fn runoff_with_args() -> ErrStr<()> {
   let args = get_args();
   if let [auth, dt, pool_str] = args.as_slice() {
      let date = parse_date(&dt)?;
      let quotes = fetch_quotes(&date).await?;
      let pool = pool_from_str(pool_str)?;
      let comp = fetch_available_assets(&auth, &quotes, &pool).await?;
      println!("Available assets for {} pivot pool are:
{}", pool_name(&pool), comp.as_csv());
      Ok(())
   } else {
      usage()
   }
}


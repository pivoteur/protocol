use chrono::NaiveDate;

use book::{
   csv_utils::CsvWriter,
   date_utils::parse_date,
   err_utils::ErrStr,
   string_utils::s,
   utils::{ get_args, get_env }
};

use libs::{
   fetchers::{assets::pool::fetch_available_assets,quotes::fetch_quotes},
   types::{ comps::Composition, pools::{ Pool, pool_from_str } }
};

fn version() -> String { s("2.01") }
fn app_name() -> String { s("aurora") }
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
   Err(s("Needs arguments <protocol> <date> <path>"))
}

pub async fn runoff_with_args() -> ErrStr<()> {
   let args = get_args();
   if let [auth, dt, pool_str] = args.as_slice() {
      let date = parse_date(&dt)?;
      generate_available_assets_report(&auth, &date, &pool_str).await
   } else {
      usage()
   }
}

async fn generate_available_assets_report(auth: &str, date: &NaiveDate,
                                          pool_str: &str) -> ErrStr<()> {
   let root_url = get_env(&format!("{}_URL", auth.to_uppercase()))?;
   let quotes = fetch_quotes(&date).await?;
   let pool = pool_from_str(pool_str)?;
   let assets = fetch_available_assets(&root_url, &quotes, &pool).await?;
   report_assets_available(&pool, &assets)
}

fn report_assets_available(pool: &Pool, comp: &Composition)-> ErrStr<()> {
   println!("Available assets for {pool} pivot pool are:
{}", comp.as_csv());
   Ok(())
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, date_utils::yesterday, utils::now };

   create_testing!("quiz07::c_open", "", true);

   run!("generate_available_assets_report", {
      let yday = yesterday();
      let _ = now(generate_available_assets_report("pivot", &yday, "btc-eth"))?;
   });
}

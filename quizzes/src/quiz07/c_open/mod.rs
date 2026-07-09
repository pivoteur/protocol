use chrono::NaiveDate;
use clap::Parser;

use book::{
   parse_args_add_banner,
   cli_utils::add_banner,
   csv_utils::CsvWriter,
   err_utils::ErrStr,
   string_utils::UppercaseString,
   utils::get_env
};

use libs::{
   fetchers::{assets::pool::fetch_available_assets,quotes::fetch_quotes},
   types::{ comps::Composition, pools::{ Pool, pool_from_str } }
};

/// Creates virtual pivots based upon available assets.
#[derive(Debug, Parser)]
struct Args {

   /// Protocol to open virtual pivots, e.g.: PIVOT
   protocol: UppercaseString,

   /// Date to run the pivot-pool scan, e.g.: $LE_DATE
   date: NaiveDate,

   /// path of pivot pool, e.g.: data/pivots/open/raw/btc-eth.tsv
   path: String,

   /// print debugging information
   #[arg(short, long)]
   debug: bool
}

pub async fn runoff_with_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   generate_available_assets_report(&args.protocol, &args.date,
                                    &args.path, args.debug).await
}

async fn generate_available_assets_report(auth: &str, date: &NaiveDate,
                                          pool_str: &str, debug: bool)
      -> ErrStr<()> {
   let root_url = get_env(&format!("{}_URL", auth.to_uppercase()))?;
   let quotes = fetch_quotes(&date).await?;
   let pool = pool_from_str(pool_str)?;
   let assets =
      fetch_available_assets(&root_url, &quotes, &pool, debug).await?;
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

   create_testing!("quiz07::c_open");

   run!("generate_available_assets_report", {
      let yday = yesterday();
      let _ =
         now(generate_available_assets_report("pivot", &yday,
                                              "btc-eth", true))?;
   });
}

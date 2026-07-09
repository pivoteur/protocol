use chrono::NaiveDate;
use clap::Parser;

use book::{
   parse_args_add_banner,
   cli_utils::add_banner,
   err_utils::ErrStr,
   string_utils::UppercaseString,
   utils::get_env
};

use libs::{
   fetchers::{
      quotes::fetch_quotes,
      assets::pool::fetch_assets,
      pool_names::fetch_pool_names,
      pivots::fetch_open_pivots
   },
   reports::print_table,
};

#[derive(Parser,Debug)]
/// Computes available assets to pivot.
struct Args {

   /// dapp protocol, e.g. PIVOT
   protocol: UppercaseString,

   /// to check availability
   date: NaiveDate,

   /// minimum pivot amount
   #[arg(short, long, default_value_t = 1000.0)]
   min: f32,

   /// show debugging information
   #[arg(short, long)]
   debug: bool
}

pub async fn runoff_with_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   list_quotes_and_assets(&args.protocol, args.date, args.min, args.debug).await
}

async fn list_quotes_and_assets(protocol: &str, date: NaiveDate, 
                                _min: f32, debug: bool) -> ErrStr<()> {
   let root_url = get_env(&format!("{protocol}_URL"))?;
   let quotes = fetch_quotes(&date).await?;
   let aliases = &quotes.aliases.clone();
   print_table("Quotes:", &[quotes]);
   let pool_names = fetch_pool_names(&root_url).await?;
   for pool in pool_names {
      let pn = pool.pool_name();
      let comp = fetch_assets(&root_url, &pool, aliases, debug).await?;
      print_table(&format!("Pool {}:", pn), &[comp]);
      let (open_pivs, _) =
         fetch_open_pivots(&root_url, &pool, aliases, debug).await?;
      print_table("Open Pivots:", &open_pivs);
   }
   Ok(())
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{
      create_testing,
      date_utils::yesterday,
      utils::now
   };

   create_testing!("quiz07::a_ssets");

   run!("list_quotes_and_assets", {
      let yday = yesterday();
      let _ =
         now(list_quotes_and_assets("PIVOT", yday, 0.0, true));
   });
}


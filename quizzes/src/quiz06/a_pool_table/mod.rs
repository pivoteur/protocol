use clap::Parser;

use book::{
   parse_args_add_banner,
   cli_utils::add_banner,
   err_utils::ErrStr,
   string_utils::UppercaseString,
   utils::get_env
};
use libs::{
   fetchers::assets::pool::fetch_assets,
   reports::print_table,
   types::{aliases::aliases,pools::{Pool,mk_pool}}
};

// as this function only calls a library function, it's not testable:
// it's infrastructure.
async fn fetch_pool_assets(auth: &str, pool: &Pool, debug: bool)
      -> ErrStr<()> {
   let aliases = aliases();
   let root = get_env(&format!("{auth}_URL"))?;
   let og = fetch_assets(&root, pool, &aliases, debug).await?;
   print_table(&format!("{} assets", pool.pool_name()), &[og]);
   Ok(())
}

/// Shows the assets of the selected pivot pool.
#[derive(Debug, Parser)]
struct Args {
   /// Protocol to compute assets, e.g.: PIVOT
   protocol: UppercaseString,

   /// primary asset of pivot pool to analyze
   primary: String,

   /// pivot asset of pivot pool to analyze
   pivot: String,

   /// print debugging information
   #[arg(short, long)]
   debug: bool
}

pub async fn runoff_get_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   let pool = mk_pool(&args.primary, &args.pivot);
   fetch_pool_assets(&args.protocol, &pool, args.debug).await
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {

   use super::*;
   use paste::paste;
   use book::{ create_testing, utils::now };

   create_testing!("quiz06::a_pool_table");

   run!("fetch_pool_assets",
        now(fetch_pool_assets("pivot", &mk_pool("btc", "eth"), true)));
}


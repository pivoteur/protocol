use clap::Parser;

use book::{
   not_implemented,
   parse_args_add_banner,
   cli_utils::add_banner,
   err_utils::ErrStr,
   num::floats::comma_floats::CommaFloat,
   string_utils::UppercaseString,
   utils::get_env
};

use libs::{
   fetchers::calls::fetch_calls,
   types::util::Id
};

/// From a trade that closes a pivot, write out the close pivot transaction
#[derive(Debug, Parser)]
#[command(name = "wyrd")]
#[command(version = "2.00")]
struct Args {
   /// protocol where the pivot is closed, e.g.: PIVOT
   protocol: UppercaseString,

   /// path to close pivot tables, e.g.: data/pivots/close/raw
   path: String,

   /// the call index for the close pivot, e.g.: 2
   ix: Id,

   /// Actual amount swapped-to, e.g.: 100.73
   amount: CommaFloat,

   /// prints debugging information
   #[arg(short, long)]
   debug: bool
}

pub async fn runoff_with_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   let root_url = get_env(&format!("{}_URL", args.protocol))?;
   let calls = fetch_calls(&root_url).await?;
   not_implemented!("f_wyrd_jr::runoff_with_args", calls)
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use paste::paste;
   use super::*;
   use book::{ create_testing, utils::now };

   create_testing!("quizzes::quiz08::f_wyrd_jr");

   run!("convert", now(runoff_with_args()));
}


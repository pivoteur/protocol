use clap::Parser;

use book::{
   parse_args_add_banner,
   cli_utils::generate_banner,
   csv_utils::as_csv,
   err_utils::ErrStr,
   num::floats::comma_floats::CommaFloat,
   string_utils::UppercaseString,
   utils::get_env
};

use libs::{
   fetchers::calls::fetch_call_data,
   processors::calls::transform_to_close,
   types::util::Id
};

/// From a trade that closes a pivot, write out the close pivot transaction
#[derive(Debug, Parser)]
#[command(name = "wyrd")]
#[command(version = "2.01")]
struct Args {
   /// protocol where the pivot is closed, e.g.: PIVOT
   protocol: UppercaseString,

   /// path to close pivot tables, e.g.: data/pivots/close/raw
   path: String,

   /// the call index for the close pivot, e.g.: 2
   ix: Id,

   /// The transaction ID of the swap, e.g.: https://snowtrace.blah.blah.blah
   tx_id: String,

   /// Actual amount swapped-to, e.g.: 100.73
   amount: CommaFloat,

   /// prints debugging information
   #[arg(short, long)]
   debug: bool
}

pub async fn runoff_with_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   let amt: f32 = args.amount.into();
   runoff_continuation(&args.protocol, &args.path, args.ix, &args.tx_id,
                       amt, args.debug).await
}

async fn runoff_continuation(protocol: &str, path: &str, ix: Id, tx_id: &str,
                             amt: f32, debug: bool) -> ErrStr<()> {
   let root_url = get_env(&format!("{}_URL", protocol))?;
   let (call, _open_pivots) = fetch_call_data(&root_url, ix, debug).await?;
   let close = transform_to_close(&call, tx_id, amt);
   let pool_path = format!("{path}/{}.tsv", &call.pool.file_name());
   println!("{}{pool_path}", as_csv(&[close], true)?);
   Ok(())
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use paste::paste;
   use super::*;
   use book::{ create_testing, utils::now };

   create_testing!("quizzes::quiz08::f_wyrd_jr");

   run!("wyrd",
        now(runoff_continuation("PIVOT", "path", 1, "asdf", 1.8e7, true)));
}


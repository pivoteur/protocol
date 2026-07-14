use clap::Parser;

use book::{
   err_utils::ErrStr,
   num::floats::comma_floats::CommaFloat,
   string_utils::UppercaseString
};

use libs::{
   fetchers::calls::fetch_calls;
   types::{
      calls::Call,
      pivots::close::ClosePivot,
      util::Id
   }
};

/// From a trade that closes a pivot, write out the close pivot transaction
#[derive(Debug, Parser)]
#[command(name = "wyrd")]
#[command(version = 2.00")]
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



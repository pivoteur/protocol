use chrono::NaiveDate;
use clap::Parser;

use serde_json;

use book::{
   parse_args_add_banner,
   cli_utils::add_banner,
   err_utils::{ ErrStr, err_or },
};

use libs::fetchers::quotes::fetch_quotes;

async fn print_quotes_as_json(dt: &NaiveDate) -> ErrStr<String> {
   let quotes = fetch_quotes(dt).await?;
   let json = err_or(serde_json::to_string_pretty(&quotes),
                    "unable to convert quotes to JSON")?;
   Ok(json)
}

/// fetches the quotes for the protocol
#[derive(Debug, Parser)]
#[command(name = "quotes")]
#[command(version = "1.01")]
struct Args {
   /// Date to run protocol-quotes
   date: NaiveDate
}

pub async fn runoff_with_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   let quotes = print_quotes_as_json(&args.date).await?;
   println!("{quotes}");
   Ok(())
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, date_utils::yesterday, utils::now };

   create_testing!("quizzes::quiz10::d_quotes");

   run!("print_quotes", {
      let json = now(print_quotes_as_json(&yesterday()))?;
      println!("Quotes as JSON:\n\n{json}");
   });
}


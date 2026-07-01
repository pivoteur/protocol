use chrono::NaiveDate;

use serde_json;

use book::{
   err_utils::{ ErrStr, err_or },
   date_utils::parse_date,
   string_utils::s,
   utils::get_args
};

use libs::fetchers::quotes::fetch_quotes;

fn version() -> String { s("1.00") }
fn app_name() -> String { s("quotes") }

fn usage() -> ErrStr<()> {
   let app = app_name();
   println!("{}, version: {}

usage:

$ {} <date>

where:
* <date> is the date of the crypto quotes
", app, version(), app);
   Err(s("missing <date> argument"))
}

async fn print_quotes_as_json(dt: &NaiveDate) -> ErrStr<String> {
   let quotes = fetch_quotes(dt).await?;
   let json = err_or(serde_json::to_string_pretty(&quotes),
                    "unable to convert quotes to JSON")?;
   Ok(json)
}

pub async fn runoff_with_args() -> ErrStr<()> {
   let args = get_args();
   if let Some(date) = args.first() {
      let dt = parse_date(&date)?;
      let quotes = print_quotes_as_json(&dt).await?;
      println!("{quotes}");
      Ok(())
   } else {
      usage()
   }
}


// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, date_utils::yesterday, utils::now };

   create_testing!("quizzes::quiz10::d_quotes", "", true);

   run!("print_quotes", {
      let json = now(print_quotes_as_json(&yesterday()))?;
      println!("Quotes as JSON:\n\n{json}");
   });
}


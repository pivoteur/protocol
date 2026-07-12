use chrono::NaiveDate;

use book::{
   date_utils::parse_date,
   err_utils::ErrStr,
   list_utils::tail,
   num_utils::parse_num,
   parse_utils::parse_str,
   table_utils::{cols,ingest,row}
};

use super::utils::fetch_lines;
use crate::{ paths::quotes_url, types::quotes::{ Quotes, mk_quotes } };

// ----- QUOTES -------------------------------------------------------

/// fetch the quotes for date; historical quote-data is reposed in git
pub async fn fetch_quotes(date: &NaiveDate) -> ErrStr<Quotes> {
   let lines = fetch_lines(&quotes_url()).await?;
   let body: Vec<String> = tail(&lines);
   fn capitalize(s: &str) -> ErrStr<String> { Ok(s.to_uppercase()) }
   let table = ingest(parse_date, capitalize, parse_str, &body, ",")?;
   if let Some(quotes_row) = row(&table, date) {
      let mut quotes = Vec::new();
      let hdrs = cols(&table);
      for (n, h) in hdrs.iter().enumerate() {
         let qt: f32 = parse_num(&quotes_row[n])?;
         quotes.push((h.as_str(), qt));
      }
      Ok(mk_quotes(date, &quotes))
   } else {
      Err(format!("Unable to find quotes for date {date}"))
   }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{
      create_testing,
      csv_utils::CsvWriter,
      date_utils::yesterday,
      utils::now
   };

   create_testing!("fetchers::quotes");

   run!("fetch_quotes", {
      let qts = now(fetch_quotes(&yesterday()))?;
      println!("Quotes are:\n{}", qts.as_table().as_csv());
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use book::date_utils::yesterday;

   #[tokio::test]
   async fn test_fetch_quotes_ok() {
      let yday = yesterday();
      let ans = fetch_quotes(&yday).await;
      assert!(ans.is_ok());
   }

   #[tokio::test]
   async fn test_fetch_quotes() -> ErrStr<()> {
      let yday = yesterday();
      let ans = fetch_quotes(&yday).await?;
      assert!(ans.lookup("BTC")? > 0.0);
      Ok(())
   }
}


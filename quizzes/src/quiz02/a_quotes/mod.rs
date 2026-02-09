use std::collections::HashMap;

use chrono::NaiveDate;

use book::{
   date_utils::parse_date,
   err_utils::ErrStr,
   list_utils::tail,
   num_utils::parse_num,
   rest_utils::read_rest,
   table_utils::{ingest,cols,rows,row},
   utils::pred
};

fn lg_raw_url() -> String {
   "https://raw.githubusercontent.com/logicalgraphs/crypto-n-rust".to_string()
}

fn quotes_url() -> String {
   format!("{}/refs/heads/main/data-files/csv/quotes.csv", lg_raw_url())
}

fn parse_str(s: &str) -> ErrStr<String> { Ok(s.to_string()) }

async fn parse_quotes() -> ErrStr<(HashMap<String, f32>, NaiveDate)> {
   let url = quotes_url();
   let daters = read_rest(&url).await?;
   let lines: Vec<String> =
      daters.lines()
            .filter_map(|l| pred(!l.is_empty(), l.to_string()))
            .collect();
   let body: Vec<String> = tail(&lines);
   let table = ingest(parse_date, parse_str, parse_str, &body, ",")?;
   let dates = rows(&table);
   let max_date =
      dates.iter().max().ok_or("No max date in table".to_string())?;
   let quotes_row =
      row(&table, &max_date).ok_or(format!("No row at max_date {max_date}"))?;
   let mut quotes: HashMap<String, f32> = HashMap::new();
   let hdrs = cols(&table);
   for (n, h) in hdrs.iter().enumerate() {
      let qt: f32 = parse_num(&quotes_row[n])?;
      quotes.insert(h.clone(), qt);
   }
   Ok((quotes, max_date.clone()))
}

pub mod functional_tests {

   use super::*;

   pub async fn runoff() -> ErrStr<usize> {

      println!("quiz02: a_quotes functional test\n");

      let (quotes, max_date) = parse_quotes().await?;
      println!("Quotes are {quotes:?} for date: {}", max_date);
      Ok(1)
   }
}

#[cfg(test)]
mod tests {

   use super::*;

   #[tokio::test]
   async fn test_parse_quotes_ok() {
      let ans = parse_quotes().await;
      assert!(ans.is_ok());
   }

   #[tokio::test]
   async fn test_quote_max_date() -> ErrStr<()> {
      let (_qts, dt) = parse_quotes().await?;
      let new_year = parse_date("2026-01-01")?;
      assert!(dt > new_year);
      Ok(())
   }

   #[tokio::test]
   async fn test_btc_quote() -> ErrStr<()> {
      let (qts, _dt) = parse_quotes().await?;
      let btc = qts.get("BTC").ok_or("No quote for BTC!".to_string())?;
      assert!(*btc > 8000.0); // remember when BTC was $8k? I do.
      Ok(())
   }
}


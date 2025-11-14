use std::collections::HashMap;

use book::{
   date_utils::parse_date,
   err_utils::ErrStr,
   list_utils::tail,
   num_utils::parse_num,
   rest_utils::read_rest,
   table_utils::{ingest,cols,rows},
   utils::pred
};

fn lg_raw_url() -> String {
   "https://raw.githubusercontent.com/logicalgraphs/crypto-n-rust".to_string()
}

fn quotes_url() -> String {
   format!("{}/refs/heads/main/data-files/csv/pivots.csv", lg_raw_url())
}

fn parse_str(s: &str) -> ErrStr<String> { Ok(s.to_string()) }

async fn parse_quotes() -> ErrStr<HashMap<String, f32>> {
   let url = quotes_url();
   let daters = read_rest(&url).await?;
   let lines: Vec<String> =
      daters.lines()
            .filter_map(|l| pred(!l.is_empty(), l.to_string()))
            .collect();
   let body: Vec<String> = tail(&lines);
   let table = ingest(parse_date, parse_str, parse_str, &body, ",")?;
   let dates = rows(&table);
   if let Some(max_date) = dates.iter().max() {
      if let Some(quotes_row) = table.data.last() {
         let mut quotes: HashMap<String, f32> = HashMap::new();
         let hdrs = cols(&table);
         for (n, h) in hdrs.iter().enumerate() {
            let qt: f32 = parse_num(&quotes_row[n])?;
            quotes.insert(h.clone(), qt);
         }
         println!("Quotes are {quotes:?} for date: {}", max_date);
         Ok(quotes)
      } else {
         Err("No last row of quotes in the table".to_string())
      }
   } else {
      Err("No max date in table".to_string())
   }
}

#[tokio::main]
async fn main() -> ErrStr<()> {
   let _quotes = parse_quotes().await?;
   Ok(())
}

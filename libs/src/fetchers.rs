/// fetch data from REST endpoints

use std::collections::HashMap;

use chrono::NaiveDate;

use book::{
   date_utils::parse_date,
   err_utils::ErrStr,
   list_utils::tail,
   num_utils::parse_num,
   rest_utils::read_rest,
   table_utils::{cols,row,ingest},
   utils::pred
};
use crate::{
   parsers::{parse_str,enum_headers},
   paths::{open_pivot_path,quotes_url},
   tables::index_table,
   types::{
      pivots::{Pivot,parse_pivot,active},
      quotes::{Quotes,mk_quotes}
   }
};

/// Fetch the pivots for pivot pool A+B; open pivots are reposed in git
pub async fn fetch_pivots(primary: &str, pivot: &str)
      -> ErrStr<(Vec<Pivot>, Vec<Pivot>)> {
   let pri = primary.to_lowercase();
   let seggs = pivot.to_lowercase();
   let url = open_pivot_path(&pri, &seggs);
   let lines = fetch_lines(&url).await?;
   let table = index_table(lines)?;

   let hdrs = enum_headers(cols(&table));

   let mut acts: Vec<Pivot> = Vec::new();
   let mut pass: Vec<Pivot> = Vec::new();

   for row in table.data {
      let piv = parse_pivot(&hdrs, &row)?;
      if active(&piv) {
         acts.push(piv.clone());
      } else {
         pass.push(piv);
      }
   }
   Ok((acts, pass))
}

/// Filter to only the open pivots for pivot pool A+B
pub async fn fetch_open_pivots(primary: &str, pivot: &str)
      -> ErrStr<Vec<Pivot>> {
   let (ans, _) = fetch_pivots(primary, pivot).await?;
   Ok(ans)
}

async fn fetch_lines(url: &str) -> ErrStr<Vec<String>> {
   let daters = read_rest(url).await?;
   let lines: Vec<String> =
      daters.lines()
            .filter_map(|l| pred(!l.is_empty(), l.to_string()))
            .collect();
   Ok(lines)
}

/// fetch the quotes for date; historical quote-data is reposed in git
pub async fn fetch_quotes(date: &NaiveDate) -> ErrStr<Quotes> {
   let lines = fetch_lines(&quotes_url()).await?;
   let body: Vec<String> = tail(&lines);
   let table = ingest(parse_date, parse_str, parse_str, &body, ",")?;
   if let Some(quotes_row) = row(&table, date) {
      let mut quotes = HashMap::new();
      let hdrs = cols(&table);
      for (n, h) in hdrs.iter().enumerate() {
         let qt: f32 = parse_num(&quotes_row[n])?;
         quotes.insert(h.clone(), qt);
      }
      Ok(mk_quotes(date.clone(), quotes))
   } else {
      Err(format!("Unable to find quotes for date {date}"))
   }
}


use book::{
   err_utils::ErrStr,
   parse_utils::{parse_id,parse_str},
   table_utils::ingest
};

use super::utils::fetch_lines;

use crate::{ paths::pivots_dir, tables::IxTable };

// ----- CALLS -------------------------------------------------------

pub async fn fetch_calls(root_url: &str) -> ErrStr<IxTable> {
   let calls_url = format!("{}/calls.csv", pivots_dir(root_url));
   let lines = fetch_lines(&calls_url).await?;
   ingest(parse_id, parse_str, parse_str, &lines, ",")
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{
      create_testing,
      csv_utils::CsvWriter,
      utils::now
   };
   use crate::fetchers::test_helpers::test_functions::marshall;

   create_testing!("fetchers::calls");

   run!("fetch_calls", {
      let (root_url, _aliases) = marshall()?;
      let calls = now(fetch_calls(&root_url))?;
      println!("\tcalls are:\n{}", calls.as_csv());
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use crate::fetchers::test_helpers::test_functions::marshall;
   use book::{
      currency::usd::USD,
      date_utils::{parse_date,yesterday},
      err_utils::err_or,
      string_utils::s,
      table_utils::{ row, val },
   };
      
   #[tokio::test]
   async fn test_fetch_calls_ok() -> ErrStr<()> {
      let (root_url, _aliases) = marshall()?;
      let mb_tab = fetch_calls(&root_url).await;
      assert!(mb_tab.is_ok());
      Ok(())
   }

   async fn test_fetch_calls() -> ErrStr<IxTable> {
      let (root_url, _aliases) = marshall()?;
      fetch_calls(&root_url).await
   }

   #[tokio::test]
   async fn test_fetch_calls_has_calls() -> ErrStr<()> {
      let calls = test_fetch_calls().await?;
      let r1 = row(&calls, &1);
      assert!(!r1.is_none(), "No calls to test! {r1:?}");
      Ok(())
   }

   fn fetch_val(t: &IxTable, r: usize) -> impl Fn(&str) -> String {
      move |st| {
         let v = val(t, &r, &s(st));
         assert!(!v.is_none());
         v.unwrap()
      }
   }

   #[tokio::test]
   async fn test_fetch_calls_close_date() -> ErrStr<()> {
      let yday = yesterday();
      let calls = test_fetch_calls().await?;
      let f = fetch_val(&calls, 1);
      let close = f("close_date");
      let closed = parse_date(&close)?;
      assert!(closed >= yday, "{closed} is neither yesterday nor today.");
      Ok(())
   }

   #[tokio::test]
   async fn test_fetch_calls_pivot_price() -> ErrStr<()> {
      let calls = test_fetch_calls().await?;
      let f = fetch_val(&calls, 1);
      let piv_price = f("pivot_close_price");
      let pp: USD = err_or(piv_price.parse(),
                           "Cannot parse USD from {piv_price}")?;
      assert!(pp.amount > 0.0);
      Ok(())
   }
}

use book::{
   csv_utils::as_csv,
   err_utils::ErrStr,
   parse_utils::{parse_id,parse_str},
   rest_utils::read_rest,
   table_utils::ingest
};

use super::{ pivots::fetch_open_pivots, utils::fetch_lines };

use crate::{
   paths::pivots_dir,
   tables::IxTable,
   types::{ aliases::aliases, calls::{Call,parse_calls}, pivots::opens::Pivot }
};

// ----- CALLS -------------------------------------------------------

pub async fn fetch_calls_table(root_url: &str) -> ErrStr<IxTable> {
   let calls_url = format!("{}/calls.csv", pivots_dir(root_url));
   let lines = fetch_lines(&calls_url).await?;
   ingest(parse_id, parse_str, parse_str, &lines, ",")
}

pub async fn fetch_calls(root_url: &str) -> ErrStr<Vec<Call>> {
   let url = format!("{}/calls.csv", pivots_dir(root_url));
   let csv_data = read_rest(&url).await?;
   parse_calls(&csv_data)
}

pub async fn fetch_call_data(root_url: &str, ix: usize, debug: bool)
      -> ErrStr<(Call, Vec<Pivot>)> {
   let call = grab_call(&root_url, ix, debug).await?;
   if debug { println!("Examining call\n{}", as_csv(&[call.clone()])?); }
   let pool = &call.pool;
   let a = aliases();
   let (all_pivs, dt) = fetch_open_pivots(root_url, pool, &a, debug).await?;
   if debug {
      println!("Fetched {} open pivots for {pool} pool; max_date: {dt}",
               all_pivs.len());
   }
   Ok((call, all_pivs))
}

async fn grab_call(root_url: &str, ix: usize, debug: bool) -> ErrStr<Call> {
   let calls = fetch_calls(root_url).await?;
   if debug { println!("Fetched {} calls", calls.len()); }
   let call = calls.get(ix - 1).ok_or("No call found at index {ix}")?;
   Ok(call.clone())
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{
      create_testing,
      csv_utils::{CsvWriter,as_csv,enumerate_csv},
      utils::now
   };
   use crate::fetchers::test_helpers::test_functions::marshall;

   create_testing!("fetchers::calls");

   run!("fetch_calls_table", " (as table rows)", {
      let (root_url, _aliases) = marshall()?;
      let calls = now(fetch_calls_table(&root_url))?;
      println!("\tcalls are:\n{}", calls.as_csv());
   });

   run!("fetch_calls", " (as structures)", {
      let (root_url, _aliases) = marshall()?;
      let calls = now(fetch_calls(&root_url))?;
      println!("\tcall structuress are:\n{}", as_csv(&calls)?);
   });

   run!("fetch_call_data", {
      let (root_url, _) = marshall()?;
      let (call, pivs) = now(fetch_call_data(&root_url, 1, true))?;
      println!("The first call is:\n\n{}", as_csv(&[call])?);
      println!("The pivots are:\n\n{}", enumerate_csv(&pivs));
   });

   run!("grab_call", {
      let (root_url, _) = marshall()?;
      let call = now(grab_call(&root_url, 1, true))?;
      println!("The first call today is\n\n{}", as_csv(&[call])?);
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
   async fn test_fetch_calls_table_ok() -> ErrStr<()> {
      let (root_url, _aliases) = marshall()?;
      let mb_tab = fetch_calls_table(&root_url).await;
      assert!(mb_tab.is_ok());
      Ok(())
   }

   async fn test_fetch_calls_table() -> ErrStr<IxTable> {
      let (root_url, _aliases) = marshall()?;
      fetch_calls_table(&root_url).await
   }

   #[tokio::test]
   async fn test_fetch_calls_has_calls() -> ErrStr<()> {
      let calls = test_fetch_calls_table().await?;
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
   async fn test_fetch_calls_table_close_date() -> ErrStr<()> {
      let yday = yesterday();
      let calls = test_fetch_calls_table().await?;
      let f = fetch_val(&calls, 1);
      let close = f("close_date");
      let closed = parse_date(&close)?;
      assert!(closed >= yday, "{closed} is neither yesterday nor today.");
      Ok(())
   }

   #[tokio::test]
   async fn test_fetch_calls_table_pivot_price() -> ErrStr<()> {
      let calls = test_fetch_calls_table().await?;
      let f = fetch_val(&calls, 1);
      let piv_price = f("pivot_close_price");
      let pp: USD = err_or(piv_price.parse(),
                           "Cannot parse USD from {piv_price}")?;
      assert!(pp.amount() > 0.0);
      Ok(())
   }
}

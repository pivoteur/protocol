use book::{ err_utils::ErrStr, rest_utils::read_rest, utils:: pred };
use crate::types::aliases::aliases;

// ----- UTILITY FUNCTIONS --------------------------------------------------

pub async fn fetch_lines(url: &str) -> ErrStr<Vec<String>> {
   let daters = read_rest(url).await?;
   let lines: Vec<String> =
      daters.lines()
      .filter_map(|l| pred(!l.is_empty(), l.to_string()))
      .collect();
   Ok(lines)
}

pub fn enlowerify(primary: &str, pivot: &str) -> (String, String) {
   (primary.to_lowercase(), pivot.to_lowercase())
}

pub fn enupperify(primary: &str, pivot: &str) -> (String, String) {
   let a = aliases();
   (a.alias(primary), a.alias(pivot))
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{
      create_testing,
      list_utils::take,
      utils::{now,debug}
   };
   use crate::paths::quotes_url;

   create_testing!("fetchers::utils");

   run!("fetch_lines", " (fetching the quotes)", {
      let qts = now(fetch_lines(&quotes_url()))?;
      println!("\tSome quotes:\n{}", take(5, &qts).join("\n"));
   });
   run_with!("enlowerify", enlowerify("BTC", "ETH"), debug);
   run_with!("enupperify", enupperify("btc", "usdc"), debug);
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use crate::paths::quotes_url;
   use book::string_utils::s;

   #[tokio::test]
   async fn test_fetch_lines_ok() {
      let ans = fetch_lines(&quotes_url()).await;
      assert!(ans.is_ok());
   }

   #[tokio::test]
   async fn test_fetch_lines() -> ErrStr<()> {
      let ans = fetch_lines(&quotes_url()).await?;
      assert!(!ans.is_empty());
      Ok(())
   }

   #[tokio::test]
   async fn fail_fetch_lines() {
      let ans = fetch_lines("READYOU.md").await;
      assert!(ans.is_err());
   }

   #[test] fn test_enupperify() {
      assert_eq!((s("BTC"), s("ETH")), enupperify("btc", "eth"));
   }

   #[test] fn test_enlowerify() {
      assert_eq!((s("avax"), s("undead")), enlowerify("AVAX", "UNDEAD"));
   }
}


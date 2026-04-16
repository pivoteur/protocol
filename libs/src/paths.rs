use book::{ err_utils::ErrStr, file_utils::dir_file };
use super::types::util::{Pool,mk_pool};

// ----- location of the pivot-files ----------------------------------------

fn raw_url(root_url: &str) -> String {
   format!("{}/refs/heads/main", root_url)
}

fn data_dir(root_url: &str) -> String { format!("{}/data", raw_url(root_url)) }
pub fn pivots_dir(root_url: &str) -> String {
   format!("{}/pivots", data_dir(root_url))
}

pub fn tsv_url(root_url: &str, tsv: &str) -> String {
   format!("{}/{}.tsv", data_dir(root_url), tsv)
}

fn open_pivots_url(root_url: &str) -> String {
   format!("{}/open/raw", pivots_dir(root_url))
}

fn pool_file(primary: &str, pivot: &str) -> String {
   format!("{primary}-{pivot}.tsv")
}

pub fn pool_assets_url(root_url: &str, primary: &str, pivot: &str) -> String {
   format!("{}/pools/{}", data_dir(root_url), pool_file(primary, pivot))
}

/// Resolves the pivot-assets to the open pivot pool URL
pub fn open_pivot_path(root_url: &str, primary: &str, pivot: &str) -> String {
   format!("{}/{}", open_pivots_url(root_url), pool_file(primary, pivot))
}

/// Constructing a pivot pool from a path
pub fn pivot_pool_from_file(path: &str) -> ErrStr<Pool> {
   let (_dir, file) = dir_file(&path)
         .ok_or_else(|| format!("Cannot dir_file({path})"))?;
   if file.ends_with(".tsv") && file.contains("-") {
      let split1: Vec<&str> = file.split(".").collect();
      let name = split1.first().unwrap();
      let split2: Vec<&str> = name.split("-").collect();
      let prim = split2.first().unwrap();
      let proper = split2.last().unwrap();
      Ok(mk_pool(prim, proper))
   } else {
      Err(format!("Cannot parse pool from {file}"))
   }
}

// ----- For to extract the quotes of the day ---------------------------------

fn lg_raw_url() -> String {
   "https://raw.githubusercontent.com/logicalgraphs/crypto-n-rust".to_string()
}

/// URL to pull the table of quotes reposed in git
pub fn quotes_url() -> String {
   format!("{}/refs/heads/main/data-files/csv/quotes.csv", lg_raw_url())
}

// ----- TESTS -------------------------------------------------------

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;

   fn pool_file() -> String { format!("btc-eth.tsv") }
   fn opens_path() -> String { format!("protocol/data/pivots/open/raw") }

   pub fn path_to_btc_eth_pivot_pool() -> String {
      format!("{}/{}", opens_path(), pool_file())
   }

   fn run_pivot_pool_from_file() -> ErrStr<usize> {
      println!("\npaths::pivot_pool_from_file functional test\n");
      let path = path_to_btc_eth_pivot_pool();
      println!("\tpath: {path}");
      let ans = pivot_pool_from_file(&path)?;
      println!("\tpool: {ans:?}\n");
      println!("paths::pivot_pool_from_file:...ok");
      Ok(1)
   }

   pub fn runoff() -> ErrStr<usize> {
      println!("\npaths functional tests\n");
      let a = run_pivot_pool_from_file()?;
      Ok(a)
   }

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn test_pivot_pool_from_file_ok() {
      let ans = pivot_pool_from_file(&path_to_btc_eth_pivot_pool());
      assert!(ans.is_ok());
   }

   #[test]
   fn fail_pivot_pool_from_file() {
      let ans = pivot_pool_from_file("ble-blah-blah-bleh");
      assert!(ans.is_err());
   }

   #[test]
   fn test_btc_eth_pivot_pool_from_file() -> ErrStr<()> {
      let (btc,eth) = pivot_pool_from_file(&path_to_btc_eth_pivot_pool())?;
      assert_eq!("btc", &btc);
      assert_eq!("eth", &eth);
      Ok(())
   }
}
}

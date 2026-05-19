use book::{
   err_utils::ErrStr,
   list_utils::filter_map_or,
   string_utils::{str2strf,s}
};
use super::utils::fetch_lines;
use crate::types::util::{Pool,mk_pool};

// ----- POOL NAMES --------------------------------------------------

pub async fn fetch_pool_names(root_url: &str) -> ErrStr<Vec<Pool>> {
   let url = format!("{root_url}/refs/heads/main/libs/pool-assets.js");
   let lines = fetch_lines(&url).await?;
   filter_map_or(str2strf(&pool), raw_pools(&lines))
}

fn raw_pools(lines: &[String]) -> Vec<String> {
   lines.iter()
        .map(|s| s.trim())
        .filter(|s| s.starts_with("["))
        .map(s)
        .collect()
}

fn pool(line: &str) -> ErrStr<Pool> {
   let v: Vec<&str> = line.split(",").collect();
   if v.len() == 2 || (v.len() == 3 && v[2].is_empty()) {
      Ok(mk_pool(&alphanum(v[0]), &alphanum(v[1])))
   } else {
      Err(format!("Unable to derive pool from {line}"))
   }
}
   
fn alphanum(input: &str) -> String {
    input.chars().filter(|c| c.is_alphanumeric()).collect()
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, utils::{ deref, now } };
   use crate::{
      fetchers::test_helpers::test_functions::marshall,
      types::util::pool_name
   };

   create_testing!("fetchers::pool_names");

   run!("fetch_pool_names", {
      let (root_url, _aliases) = marshall()?;
      let pool_names = now(fetch_pool_names(&root_url))?;
      let pn: Vec<String> =
         pool_names.into_iter().map(deref(pool_name)).collect();
      println!("\tpool names:\n\n\t{pn:?}");
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use crate::fetchers::test_helpers::test_functions::marshall;
   use book::tuple_utils::{fst,snd};

   fn sample_pivot_pools() -> Vec<String> { "
   // created by: pools, version: 1.00
      
const poolAssets = {
   generated: '2026-04-18',
   assets: [
      [ 'AVAX', 'UNDEAD' ],
      [ 'BTC', 'AVAX' ],
      [ 'BTC', 'ETH' ],
      [ 'BTC', 'UNDEAD' ],
      [ 'BTC', 'USDC' ],
      [ 'ETH', 'UNDEAD' ],
      [ 'UNDEAD', 'USDC' ]
   ]
};".lines().map(s).collect()
   }

   #[test]
   fn test_raw_pivots_count() {
      let rp = raw_pools(&sample_pivot_pools());
      assert_eq!(7, rp.len());
   }

   #[test]
   fn test_pool_last_ok() {
      let mb_btc_eth = pool("[ 'BTC', 'ETH' ]");
      assert!(mb_btc_eth.is_ok());
   }

   #[test]
   fn fail_non_pool() {
      let mb_btc_eth = pool("The gnerrs com from der voodverk out.");
      assert!(mb_btc_eth.is_err());
   }

   #[test]
   fn fail_pool_trailing_detritus() {
      let mb_btc_eth = pool("    [ 'BTC', 'ETH' ], and some other data");
      assert!(mb_btc_eth.is_err());
   }

   #[test]
   fn test_pool_btc_eth_medial() -> ErrStr<()> {
      let btc_eth = pool("[ 'BTC', 'ETH' ],")?;
      assert_eq!("BTC", &fst(btc_eth.clone()));
      assert_eq!("ETH", &snd(btc_eth));
      Ok(())
   }

   #[tokio::test]
   async fn test_fetch_pool_names_ok() -> ErrStr<()> {
      let (root_url, _aliases) = marshall()?;
      let ans = fetch_pool_names(&root_url).await;
      assert!(ans.is_ok());
      Ok(())
   }

   #[tokio::test]
   async fn test_fetch_pool_names_has_pools() -> ErrStr<()> {
      let (root_url, _aliases) = marshall()?;
      let ans = fetch_pool_names(&root_url).await?;
      assert!(!ans.is_empty());
      Ok(())
   }

   #[tokio::test]
   async fn test_fetch_pool_names_has_btc_pool() -> ErrStr<()> {
      let (root_url, _aliases) = marshall()?;
      let pools = fetch_pool_names(&root_url).await?;
      assert!(pools.iter().any(|(prim,_)| prim == "BTC"));
      Ok(())
   }
}

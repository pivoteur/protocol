use std::collections::HashMap;

use chrono::NaiveDate;

use book::{
   not_implemented,
   err_utils::ErrStr, 
   utils::get_env
};

use super::{
   fetchers::{
      assets::pool::fetch_available_assets,
      pivots::fetch_pivots,
      pool_names::fetch_pool_names,
      quotes::fetch_quotes,
      wallets::fetch_wallets_balances,
      whitelist::fetch_whitelist
   },
   reports::{Proposal,mk_proposal},
   types::{
      measurable::sort_descending,
      pivots::{Pivot,next_close_id,partition_on},
      pools::Pool,
      proposals::proposes::{Propose,propose as propose_f},
      tokens::allocations::{
         pools::Pools
      },
      util::Token
   }
};

// ---- PROPOSALS -------------------------------------------------------

pub async fn process_pools(auth_name: &str, date: &NaiveDate, debug: bool)
      -> ErrStr<(Vec<Proposal>, Vec<Pool>)> {
   let auth = auth_name.to_uppercase();
   let root_url = get_env(&format!("{auth}_URL"))?;
   let pools = fetch_pool_names(&root_url).await?;
   process_pools0(&root_url, &pools, date, debug).await
}

async fn process_pools0(root_url: &str, pools: &Vec<Pool>,
      date: &NaiveDate, debug: bool) -> ErrStr<(Vec<Proposal>, Vec<Pool>)> {
   let quotes = fetch_quotes(&date).await?;
   let a = &quotes.aliases;
   let proposer = propose_f(&quotes);
   let mut no_closes = Vec::new();
   let mut proposals = Vec::new();

   for pool in pools {
      if debug { println!("Processing pool {pool}"); }
      let (primary, _) = pool.as_tuple();
      let ((opens, closes), max_date) =
         fetch_pivots(root_url, pool, a).await?;
      let ans = propose(&proposer, pool, &primary, opens, closes, &max_date)?;
      if ans.is_empty() {
         no_closes.push(pool.clone());
      } else {
         proposals.extend(ans); 
      }
   }
   proposals.sort_by(sort_descending);
   Ok((proposals, no_closes))
}

type Ixs<A> = (Vec<A>, usize);
type Ix<A> = (A, usize);

fn propose(proposer: impl Fn(Ixs<Pivot>) -> ErrStr<Option<Ix<Propose>>>,
           pool: &Pool, prim: &Token, opens: Vec<Pivot>, closes: Vec<Pivot>,
           max_date: &NaiveDate) -> ErrStr<Vec<Proposal>> {
   let next_close = next_close_id(&closes);
   let len = &opens.len();
   let (lefts, rights) = partition_on(prim, opens);
   let (follow, mut props) =
      if let Some((prop, nxt)) = proposer((lefts, next_close))? {
         (nxt, vec![mk_proposal(&pool, &max_date, *len, prop)])
      } else {
         (next_close, Vec::new())
   };
   let _ = proposer((rights, follow))?
      .and_then(|(prop, _)| {
         props.push(mk_proposal(&pool, &max_date, *len, prop));
         Some(1)
      });
   Ok(props)
}

// ----- Pool Assets -------------------------------------------------

pub async fn process_pool_assets(root_url: &str, dt: &NaiveDate)
      -> ErrStr<Pools> {
   let pool_names = fetch_pool_names(root_url).await?;
   let quotes = fetch_quotes(dt).await?;
/*
   let pools: Vec<PoolAssets> =
      async_filter_map(process_each_pool_assets::<Future>(root_url, &quotes), pool_names).await?;
*/
   not_implemented!("process_pool_assets", pool_names, quotes)
}

/*
fn process_each_pool_assets<F>(root_url: &str, q: &Quotes)
      -> impl Fn(Pool) -> F where F: Future<Output = ErrStr<PoolAssets>> {
   move |p: Pool| async {
      let (assets, opens) =
         fetch_assets_and_open_pivots(root_url, q, &p).await?;
   }
   not_implemented!("process_each_pool_assets", root_url, q)
}
*/

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, date_utils::yesterday, utils::now };
   use crate::reports::print_table;

   create_testing!("processors");

   run!("process_pools", {
      let yday = yesterday();
      let (calls,nixen) = now(process_pools("pivot", &yday, true))?;
      let hdr = format!("Calls for {}:\n", yday);
      print_table(&hdr, &calls);
      let ps: Vec<String> = nixen.iter().map(Pool::pool_name).collect();
      println!("Pools with no calls:\n\n{}", ps.join("\t"));
   });

   run!("compute_health", {
      let yday = yesterday();
      let comps = now(compute_health("pivot", &yday, true))?;
      report_health(yday, comps);
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use std::collections::HashSet;
   use super::*;
   use crate::fetchers::test_helpers::test_functions::marshall;
   use book::{ date_utils::yesterday, utils::get_env };

   #[tokio::test]
   async fn fail_process_pools() {
      let ans = process_pools("asdf", &yesterday(), true).await;
      assert!(ans.is_err());
   }

   #[tokio::test]
   async fn test_process_pools_ok() {
      let ans = process_pools("pivot", &yesterday(), true).await;
      assert!(ans.is_ok());
   }

   #[tokio::test]
   async fn test_process_pools_all_pools_considered() -> ErrStr<()> {
      let (root_url, _aliases) = marshall()?;
      let pools = fetch_pool_names(&root_url).await?;
      let (calls, neins) = process_pools("pivot", &yesterday(), true).await?;
      let npools = pools.len();
      let cnn = calls.len() + neins.len();
      assert!(cnn >= npools,
              "Number of pools: {npools}; calls and no-calls: {cnn}");
      Ok(())
   }

   #[tokio::test] async fn fail_process_wallet_balances() {
      let ans = process_wallet_balances("asdf", false).await;
      assert!(ans.is_err());
   }

   #[tokio::test] async fn test_process_wallet_balances_ok() {
      let ans = process_wallet_balances("pivot", true).await;
      assert!(ans.is_ok());
   }

   #[tokio::test] async fn test_process_wallet_balances_has_values()
         -> ErrStr<()> {
      let ans = process_wallet_balances("pivot", true).await?;
      assert!(!ans.is_empty());
     Ok(())
   }

   #[tokio::test] async fn test_compute_health_ok() {
      assert!(compute_health("pivot", &yesterday(), false).await.is_ok());
   }

   #[tokio::test] async fn test_compute_health_all_pools_with_debug()
         -> ErrStr<()> {
      let yday = yesterday();
      let auth = "PIVOT";
      let root_url = get_env(&format!("{auth}_URL"))?;
      let npools = fetch_pool_names(&root_url).await?;
      let pool_names: HashSet<String> = npools.iter().map(pool_name).collect();
      let assets = compute_health(auth, &yday, true).await?;
      let al = &assets.len();
      let pl = &pool_names.len();
      assert_eq!(pl, al, "Assets {al} do not equal pools {pl}!");
      for a in assets {
         let asset = a.pool_name();
         assert!(pool_names.contains(&asset),
                 "I do not know this pool: {asset}");
      }
      Ok(())
   }
}


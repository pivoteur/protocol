use chrono::NaiveDate;

use book::{
   err_utils::ErrStr, 
   utils::get_env
};

use crate::{
   fetchers::{
      pivots::fetch_pivots,
      quotes::fetch_quotes,
      pool_names::fetch_pool_names
   },
   reports::{Proposal,mk_proposal},
   types::{
      measurable::sort_descending,
      pivots::{Pivot,next_close_id,partition_on},
      pools::Pool,
      proposals::proposes::{Propose,propose as propose_f},
      util::Token
   }
};

// ---- Proposals -------------------------------------------------------

pub async fn process_pools(auth_name: &str, date: &NaiveDate)
      -> ErrStr<(Vec<Proposal>, Vec<Pool>)> {
   let auth = auth_name.to_uppercase();
   let root_url = get_env(&format!("{auth}_URL"))?;
   let pools = fetch_pool_names(&root_url).await?;
   process_pools0(&root_url, &pools, date).await
}

async fn process_pools0(root_url: &str, pools: &Vec<Pool>, date: &NaiveDate)
      -> ErrStr<(Vec<Proposal>, Vec<Pool>)> {
   let quotes = fetch_quotes(&date).await?;
   let a = &quotes.aliases;
   let proposer = propose_f(&quotes);
   let mut no_closes = Vec::new();
   let mut proposals = Vec::new();

   for pool in pools {
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

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, date_utils::yesterday, utils::now };
   use crate::reports::print_table;

   create_testing!("processors");

   run!("process_pools", {
      let yday = yesterday();
      let (calls,nixen) = now(process_pools("pivot", &yday))?;
      let hdr = format!("Calls for {}:\n", yday);
      print_table(&hdr, &calls);
      let ps: Vec<String> = nixen.iter().map(Pool::pool_name).collect();
      println!("Pools with no calls:\n\n{}", ps.join("\t"));
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use crate::fetchers::test_helpers::test_functions::marshall;
   use book::date_utils::yesterday;

   #[tokio::test]
   async fn fail_process_pools() {
      let ans = process_pools("asdf", &yesterday()).await;
      assert!(ans.is_err());
   }

   #[tokio::test]
   async fn test_process_pools_ok() {
      let ans = process_pools("pivot", &yesterday()).await;
      assert!(ans.is_ok());
   }

   #[tokio::test]
   async fn test_process_pools_all_pools_considered() -> ErrStr<()> {
      let (root_url, _aliases) = marshall()?;
      let pools = fetch_pool_names(&root_url).await?;
      let (calls, neins) = process_pools("pivot", &yesterday()).await?;
      let npools = pools.len();
      let cnn = calls.len() + neins.len();
      assert!(cnn >= npools,
              "Number of pools: {npools}; calls and no-calls: {cnn}");
      Ok(())
   }
}


use chrono::NaiveDate;

use book::{
   date_utils::parse_date,
   err_utils::ErrStr,
   utils::get_env
};

use crate::{
   types::{
      measurable::sort_descending,
      pivots::{Pivot,next_close_id,partition_on},
      proposals::proposes::{Propose,propose as propose_f},
      util::{Token,Pool}
   },
   fetchers::{fetch_pivots,fetch_quotes,fetch_pool_names},
   reports::{Proposal,mk_proposal}
};

pub async fn process_pools(auth_name: &str, dt: &str)
      -> ErrStr<(Vec<Proposal>, Vec<Pool>)> {
   let auth = auth_name.to_uppercase();
   let date = parse_date(dt)?;
   let root_url = get_env(&format!("{auth}_URL"))?;
   let pools = fetch_pool_names(&root_url).await?;
   process_pools0(&root_url, &pools, date).await
}

async fn process_pools0(root_url: &str, pools: &Vec<Pool>, date: NaiveDate)
      -> ErrStr<(Vec<Proposal>, Vec<Pool>)> {
   let quotes = fetch_quotes(&date).await?;
   let a = &quotes.aliases;
   let proposer = propose_f(&quotes);
   let mut no_closes = Vec::new();
   let mut proposals = Vec::new();

   for pool in pools {
      let (prim, piv) = pool;
      let ((opens, closes), max_date) =
         fetch_pivots(root_url, prim, piv, a).await?;
      let ans = propose(&proposer, pool, prim, opens, closes, max_date)?;
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
           max_date: NaiveDate) -> ErrStr<Vec<Proposal>> {
   let next_close = next_close_id(&closes);
   let len = &opens.len();
   let (lefts, rights) = partition_on(prim, opens);
   let (follow, mut props) =
      if let Some((prop, nxt)) = proposer((lefts, next_close))? {
         (nxt, vec![mk_proposal(&pool, max_date, *len, prop)])
      } else {
         (next_close, Vec::new())
   };
   let _ = proposer((rights, follow))?
      .and_then(|(prop, _)| {
         props.push(mk_proposal(&pool, max_date, *len, prop));
         Some(1)
      });
   Ok(props)
}

// ----- TESTS -------------------------------------------------------

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use std::fmt::Debug;
   use super::*;
   use book::{ date_utils::yesterday, string_utils::s };

   fn preambles(test: Option<&str>) -> String {
      let (t, z) = test.and_then(|t| Some((format!("::run_{t}"), "")))
                       .unwrap_or((s(""), "s"));
      println!("\nprocessors{t} functional test{z}\n");
      t
   }
   fn preamble(test: &str) -> String { preambles(Some(test)) }

   fn report<RES: Debug>(test: &str, result: RES) -> ErrStr<usize> {
      println!(
"	Result:
{result:?}

processors{test}:...ok
");
      Ok(1)
   }

   fn yday() -> String { format!("{}", yesterday()) }
   async fn run_process_pools() -> ErrStr<usize> {
      let test = preamble("process_pools");
      let calls_n_neins = process_pools("pivot", &yday()).await?;
      report(&test, calls_n_neins)
   }

   pub async fn runoff() -> ErrStr<usize> {
      preambles(None);
      let a = run_process_pools().await?;
      Ok(a)
   }

#[cfg(test)]
mod tests {
   use super::*;
   use crate::fetchers::functional_tests::marshall;

   #[tokio::test]
   async fn fail_process_pools() {
      let ans = process_pools("asdf", &yday()).await;
      assert!(ans.is_err());
   }

   #[tokio::test]
   async fn test_process_pools_ok() {
      let ans = process_pools("pivot", &yday()).await;
      assert!(ans.is_ok());
   }

   #[tokio::test]
   async fn test_process_pools_all_pools_considered() -> ErrStr<()> {
      let (root_url, _aliases) = marshall()?;
      let pools = fetch_pool_names(&root_url).await?;
      let (calls, neins) = process_pools("pivot", &yday()).await?;
      let npools = pools.len();
      let cnn = calls.len() + neins.len();
      assert!(cnn >= npools,
              "Number of pools: {npools}; calls and no-calls: {cnn}");
      Ok(())
   }
}
}


use chrono::NaiveDate;

use book::{
   date_utils::parse_date,
   err_utils::ErrStr,
   utils::get_env
};

use crate::{
   types::{
      util::Pool,
      pivots::{propose,next_close_id,partition_on}
   },
   fetchers::{fetch_pivots,fetch_quotes},
   git::fetch_pool_names,
   reports::{Proposal,mk_proposal}
};

pub async fn process_pools(auth_name: &str, dt: &str)
      -> ErrStr<(Vec<Proposal>, Vec<Pool>)> {
   let auth = auth_name.to_uppercase();
   let date = parse_date(dt)?;
   let root_url = get_env(&format!("{auth}_URL"))?;
   let pools = fetch_pool_names(&auth).await?;
   process_pools0(&root_url, &pools, date).await
}

async fn process_pools0(root_url: &str, pools: &Vec<Pool>, date: NaiveDate)
      -> ErrStr<(Vec<Proposal>, Vec<Pool>)> {
   let quotes = fetch_quotes(&date).await?;
   let proposer = propose(&quotes);
   let mut no_closes = Vec::new();
   let mut proposals = Vec::new();

   for pool in pools {
      let (prim, piv) = pool;
      let (opens, closes, max_date) = fetch_pivots(root_url, prim, piv).await?;
      let next_close = next_close_id(&closes);
      let len = &opens.len();
      let (lefts, rights) = partition_on(prim, opens);
      let mut props = false;
      let follow = if let Some((prop, nxt)) = proposer((lefts, next_close))? {
         proposals.push(mk_proposal(&pool, max_date, *len, prop));
         props = true;
         nxt
      } else {
         next_close
      };
      if let Some((prop, _)) = proposer((rights, follow))? {
         proposals.push(mk_proposal(&pool, max_date, *len, prop));
         props = true;
      }

      if !props {
         no_closes.push(pool.clone());
      }
   }
   Ok((proposals, no_closes))
}


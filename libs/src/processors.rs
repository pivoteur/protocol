use chrono::NaiveDate;

use book::{
   date_utils::parse_date,
   err_utils::ErrStr,
   utils::get_env
};

use crate::{
   types::{
      measurable::sort_descending,
      pivots::pivots::{Pivot,next_close_id,partition_on},
      proposals::proposes::{Propose,propose as propose_f},
      util::{Token,Pool}
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
   let pools = fetch_pool_names(&auth, "data/pivots/open/raw/").await?;
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
      let (opens, closes, max_date) =
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

type IxPivots = (Vec<Pivot>, usize);

fn propose(proposer: impl Fn(IxPivots) -> ErrStr<Option<(Propose, usize)>>,
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


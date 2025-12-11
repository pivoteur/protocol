use chrono::NaiveDate;

use book::err_utils::ErrStr;

use crate::{
   types::{
      util::Pool,
      pivots::{propose,next_close_id,partition_on}
   },
   fetchers::{fetch_pivots,fetch_quotes},
   reports::{print_table,header}
};

pub async fn process_pools(root_url: &str, pools: &Vec<Pool>, date: NaiveDate)
      -> ErrStr<()> {
   let quotes = fetch_quotes(&date).await?;
   let proposer = propose(&quotes);
   let mut no_closes = Vec::new();
   let mut first_time = true;
   fn printer(s: &String) { println!("{s}"); }

   for pool in pools {
      let (prim, piv) = pool;
      let (opens, closes, max_date) = fetch_pivots(root_url, prim, piv).await?;
      let next_close = next_close_id(&closes);
      let len = &opens.len();
      let (lefts, rights) = partition_on(prim, opens);
      let mut props = Vec::new();
      let follow = if let Some((prop, nxt)) = proposer((lefts, next_close))? {
         props.push(prop);
         nxt
      } else {
         next_close
      };
      if let Some((prop, _)) = proposer((rights, follow))? {
         props.push(prop);
      }

      if props.is_empty() {
         no_closes.push(pool);
      } else {
         print_table(printer, &mut first_time, 
                     prim, piv, *len, &max_date,
                     "No close pivots", &props);
      }
   }
   if !no_closes.is_empty() {
      println!("\nPivot pools with no closes:\n");
      for (prim, piv) in no_closes {
         println!("* {}", header(prim, piv));
      }
   }
   Ok(())
}

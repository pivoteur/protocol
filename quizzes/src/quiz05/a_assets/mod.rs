use book::err_utils::ErrStr;

use libs::{
   collections::assets::{mk_assets,assets_by_tvl},
   processors::process_pools,
   reports::{report_proposes,proposal,print_table,Proposal}
};

fn version() -> String { "1.10".to_string() }
fn app_name() -> String { "dusk".to_string() }

fn usage() -> ErrStr<()> {
   println!("Usage:

$ {} <protocol> <date>

where:

* <protocol> is the protocol to be analyzed, e.g.: PIVOT
* <date> is the date to propose pivots, e.g. 2025-12-18
", app_name());
   Err("Need <protocol> and <date> arguments".to_string())
}

async fn propose(auth: &str, dt: &str) -> ErrStr<usize> {
   let (proposals, no_closes) = process_pools(&auth, &dt).await?;
   report_proposes(&proposals, &no_closes);
   if !proposals.is_empty() { tokens_to_pivot(proposals); }
   Ok(1)
}

fn tokens_to_pivot(proposals: Vec<Proposal>) {
   let mut tokens = mk_assets();
   proposals.iter().for_each(|p| {
         let asset = proposal(p).pivot_amount();
         tokens.add(asset);
   });
   print_table("Assets to pivot", &assets_by_tvl(&tokens));
}

// ----- FUNCTIONAL TESTS -----------------------------------------------------

pub mod functional_tests {
   use super::*;
   use book::{ date_utils::yesterday, utils::get_args };

   pub async fn runoff_with_args() -> ErrStr<()> {
      println!("{}, version: {}", app_name(), version());
      if let [ath, dt] = get_args().as_slice() {
         let _ = propose(ath, dt).await?;
         Ok(())
      } else {
         usage()
      }
   }

   pub async fn runoff() -> ErrStr<usize> {
      let yday = format!("{}", yesterday());
      propose("pivot", &yday).await
   }
}


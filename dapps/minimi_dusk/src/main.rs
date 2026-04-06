use book::err_utils::ErrStr;

use libs::{
   collections::assets::{mk_assets,assets_by_tvl},
   processors::process_pools,
   reports::{report_proposes,proposal,print_table,Proposal}
};

fn app_name() -> String { "minimi_dusk".to_string() }

fn usage() -> ErrStr<()> {
   println!("Usage:

$ {} [--min] <protocol> <date>

where:

* <protocol> is the protocol to be analyzed, e.g.: PIVOT
* <date> is the date to propose pivots, e.g. 2025-12-18
* --min (optional) outputs only the proposals table
", app_name());
   Err("Need <protocol> and <date> arguments".to_string())
}

pub async fn propose(auth: &str, dt: &str, min: bool) -> ErrStr<usize> {
   let (proposals, no_closes) = process_pools(&auth, &dt).await?;
   if min {
      report_proposes(&proposals, &vec![]);
   } else {
      report_proposes(&proposals, &no_closes);
   }
   if !min && !proposals.is_empty() { tokens_to_pivot(proposals); }
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

// ----- TESTS -----------------------------------------------------

// ----- UNIT TESTS ------------------------------------------------

#[cfg(test)]
mod unit_tests {
   use super::*;

   #[test]
   fn test_app_name() {
      assert_eq!(app_name(), "minimi_dusk");
   }

   #[test]
   fn test_usage_returns_err() {
      let result = usage();
      assert!(result.is_err());
      assert_eq!(result.unwrap_err(), "Need <protocol> and <date> arguments");
   }

   #[test]
   fn test_tokens_to_pivot_empty() {
      tokens_to_pivot(vec![]);
   }
}

// ----- FUNCTIONAL TESTS ------------------------------------------

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use book::date_utils::yesterday;
   use book::utils::get_args;

   pub async fn runoff_with_args() -> ErrStr<()> {
      let args = get_args();
      let min = args.contains(&"--min".to_string());
      let args: Vec<String> = args.into_iter().filter(|a| a != "--min").collect();
      if let [auth, dt] = args.as_slice() {
         let _ = propose(auth, dt, min).await?;
         Ok(())
      } else {
         usage()
      }
   }

   #[tokio::test]
   pub async fn runoff() -> ErrStr<()> {
      let yday = format!("{}", yesterday());
      propose("pivot", &yday, false).await?;
      Ok(())
   }

   #[tokio::test]
   pub async fn runoff_min() -> ErrStr<()> {
      let yday = format!("{}", yesterday());
      propose("pivot", &yday, true).await?;
      Ok(())
   }
}

#[tokio::main]
async fn main() -> ErrStr<()> {
   functional_tests::runoff_with_args().await
}

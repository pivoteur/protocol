use chrono::NaiveDate;
use clap::Parser;

use book::{
   debug,
   parse_args_add_banner,
   cli_utils::generate_banner,
   err_utils::ErrStr,
   string_utils::UppercaseString
};

use libs::{
   processors::proposals::process_pools,
   collections::assets::{ assets_by_tvl, from_coins },
   reports::{ Proposal, print_table, proposal, report_proposes },
   types::tokens::coins::Coin
};

pub async fn propose(auth: &str, date: &NaiveDate, debug: bool) -> ErrStr<()> {
   debug!("propose", debug);
   log!("Processing pools for {} on date {}", auth, date);
   let (proposals, no_closes) = process_pools(auth, date, debug).await?;
   let x = if !debug { &vec![] } else { &no_closes };
   report_proposes(proposals.clone(), x, !debug);
   if debug && !proposals.is_empty() { tokens_to_pivot(proposals)?; }
   Ok(())
}

fn tokens_to_pivot(proposals: Vec<Proposal>) -> ErrStr<()> {
   let coins: Vec<Coin> =
      proposals.iter().map(|p| proposal(p).pivot_amount()).collect();
   let tokens = from_coins(&coins);
   print_table("Assets to pivot", &assets_by_tvl(&tokens));
   Ok(())
}

/// Make the close pivot call
#[derive(Debug, Parser)]
#[command(name = "dusk")]
#[command(version = "2.0.9")]
struct Args {
   /// Protocol to analyze pivots to close, e.g.: PIVOT
   protocol: UppercaseString,

   /// Date to make the calls, e.g.: $LE_DATE
   date: NaiveDate,

   /// Minimal output, suitable for updating calls.tsv
   #[arg(short, long)]
   min: bool
}

pub async fn runoff_with_args() -> ErrStr<()> {
  let args = parse_args_add_banner!(Args);
  let debug = !args.min;
  propose(&args.protocol, &args.date, debug).await
}

// ----- UNIT TESTS ------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod unit_tests {
   use super::*;
   #[test] fn test_tokens_to_pivot_empty() {
      let res = tokens_to_pivot(vec![]);
      assert!(res.is_ok());
   }
}

// ----- FUNCTIONAL TESTS ------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, date_utils::yesterday, utils::now };

   create_testing!("quiz05::b_dusk_min");

   run!("full_dusk", now(propose("pivot", &yesterday(), false))?);
   run!("dusky_min", now(propose("pivot", &yesterday(), true))?);
}

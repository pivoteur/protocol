use chrono::NaiveDate;
use clap::Parser;

use book::{
   parse_args_add_banner,
   cli_utils::add_banner,
   err_utils::ErrStr,
   string_utils::UppercaseString
};

use libs::{
   processors::process_pools,
   collections::assets::{ assets_by_tvl, mk_assets },
   reports::{ Proposal, print_table, proposal, report_proposes }
};

pub async fn propose(auth: &str, date: &NaiveDate, debug: bool) -> ErrStr<()> {
   if debug { println!("Processing pools for {auth} on date {date}"); }
   let (proposals, no_closes) = process_pools(auth, date, debug).await?;
   let x = if !debug { &vec![] } else { &no_closes };
   report_proposes(proposals.clone(), x, !debug);
   if debug && !proposals.is_empty() { tokens_to_pivot(proposals); }
   Ok(())
}

fn tokens_to_pivot(proposals: Vec<Proposal>) {
   let mut tokens = mk_assets();
   proposals.iter().for_each(|p| {
      let asset = proposal(p).pivot_amount();
      tokens.add(asset);
   });
   print_table("Assets to pivot", &assets_by_tvl(&tokens));
}

/// Make the close pivot call
#[derive(Debug, Parser)]
#[command(name = "dusk")]
#[command(version = "2.05")]
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
   #[test] fn test_tokens_to_pivot_empty() { tokens_to_pivot(vec![]); }
}

// ----- FUNCTIONAL TESTS ------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, date_utils::yesterday, utils::now };

   create_testing!("quiz05::b_dusk_min");

   run!("full_dusk", { let _ = now(propose("pivot", &yesterday(), false)); });
   run!("dusky_min", { let _ = now(propose("pivot", &yesterday(), true)); });
}

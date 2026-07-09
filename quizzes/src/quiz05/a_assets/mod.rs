use chrono::NaiveDate;
use clap::Parser;

use book::{
   parse_args_add_banner,
   cli_utils::add_banner,
   err_utils::ErrStr,
   string_utils::UppercaseString
};

use libs::{
   processors::proposals::process_pools,
   collections::assets::{ assets_by_tvl, mk_assets },
   reports::{ Proposal, print_table, proposal, report_proposes }
};

async fn propose(auth: &str, dt: &NaiveDate, debug: bool) -> ErrStr<()> {
   let (proposals, no_closes) = process_pools(&auth, &dt, debug).await?;
   report_proposes(proposals.clone(), &no_closes, false);
   if !proposals.is_empty() { tokens_to_pivot(proposals); }
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

/// Make close pivot calls
#[derive(Debug, Parser)]
struct Args {
   /// protocol to recommend pivots to close
   protocol: UppercaseString,

   /// date to make close pivot recommendations, e.g.: $LE_DATE
   date: NaiveDate,

   /// show debugging information
   #[arg(short, long)]
   debug: bool
}

pub async fn runoff_with_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   propose(&args.protocol, &args.date, args.debug).await
}

// ----- TESTS -----------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, date_utils::yesterday, utils::now };

   create_testing!("quiz05::a_assets");

   run!("propose", now(propose("pivot", &yesterday(), true)));
}

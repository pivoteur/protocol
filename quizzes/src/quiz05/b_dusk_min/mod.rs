use chrono::NaiveDate;

use book::{
   date_utils::parse_date,
   err_utils::ErrStr,
   string_utils::s,
   utils::get_args
};

use libs::{
   processors::process_pools,
   collections::assets::{ assets_by_tvl, mk_assets },
   reports::{ Proposal, print_table, proposal, report_proposes }
};

fn version() -> String { s("2.04") }
fn app_name() -> String { s("dusk") }
fn usage() -> ErrStr<()> {
    println!("Usage:

$ {} [--min] <protocol> <date>

where:
* <protocol> is the protocol to be analyzed, e.g.: PIVOT
* <date> is the date to propose pivots, e.g. 2025-12-18
* --min (optional) outputs only the proposals table", app_name());
Err(s("Need <protocol> and <date> arguments"))
}

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

pub async fn runoff_with_args() -> ErrStr<()> {
    let (_debug, args) = get_args();
    let min = args.contains("--min");
    let debug = !min;
    let args: Vec<String> =
       args.into_iter().filter(|a| a != "--min").collect();
    if debug {
        println!("\n{}, version: {}\n", app_name(), version());
    }
    if let [auth, dt] = args.as_slice() {
        let date = parse_date(&dt)?;
        propose(auth, &date, debug).await
    } else {
        usage()
    }
}

// ----- UNIT TESTS ------------------------------------------------

#[cfg(test)]
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

    create_testing!("quiz05::b_dusk_min", "", true);

    run!("full_dusk", { let _ = now(propose("pivot", &yesterday(), false)); });
    run!("dusky_min", { let _ = now(propose("pivot", &yesterday(), true)); });
}

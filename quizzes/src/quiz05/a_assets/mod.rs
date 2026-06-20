use chrono::NaiveDate;

use book::{ date_utils::parse_date, err_utils::ErrStr, utils::get_args };
use libs::{
    processors::process_pools,
    collections::assets::{ assets_by_tvl, mk_assets },
    reports::{ Proposal, print_table, proposal, report_proposes }
};

fn version() -> String { "1.11".to_string() }
fn app_name() -> String { "dusk".to_string() }

fn usage() -> ErrStr<()> {
    println!(
        "Usage:

$ {} <protocol> <date>

where:
* <protocol> is the protocol to be analyzed, e.g.: PIVOT
* <date> is the date to propose pivots, e.g. 2025-12-18", app_name());
Err("Need <protocol> and <date> arguments".to_string())
}

async fn propose(auth: &str, dt: &NaiveDate) -> ErrStr<usize> {
    let (proposals, no_closes) = process_pools(&auth, &dt).await?;
    report_proposes(proposals.clone(), &no_closes, false);
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

pub async fn runoff_with_args() -> ErrStr<()> {
    println!("{}, version: {}", app_name(), version());
    if let [ath, dt] = get_args().as_slice() {
        let date = parse_date(&dt)?;
        let _ = propose(ath, &date).await?;
        Ok(())
    } else {
        usage()
    }
}

// ----- TESTS -----------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
    use super::*;
    use paste::paste;
    use book::{ create_testing, date_utils::yesterday, utils::now };

    create_testing!("quiz05::a_assets", "", true);

    run!("propose", now(propose("pivot", &yesterday())));
}

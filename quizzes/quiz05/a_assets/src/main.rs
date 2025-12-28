use book::{
   utils::get_args,
   err_utils::ErrStr
};

use libs::{
   collections::assets::{mk_assets,assets_by_tvl},
   processors::process_pools,
   reports::{report_proposes,proposal,print_table,Proposal},
   types::pivots::pivot_amount
};

fn version() -> String { "1.08".to_string() }
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

#[tokio::main]
async fn main() -> ErrStr<()> {
   if let [ath, dt] = get_args().as_slice() {
      let (proposals, no_closes) = process_pools(&ath, &dt).await?;
      println!("{}, version: {}", app_name(), version());
      report_proposes(&proposals, &no_closes);
      if !proposals.is_empty() { tokens_to_pivot(proposals); }
      Ok(())
   } else {
      usage()
   }
}

fn tokens_to_pivot(proposals: Vec<Proposal>) {
   let mut tokens = mk_assets();
   proposals.iter().for_each(|p| {
         let asset = pivot_amount(&proposal(p));
         tokens.add(asset);
   });
   print_table("Assets to pivot", &assets_by_tvl(&tokens));
}


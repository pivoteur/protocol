// use std::collections::HashMap;

use book::{
   utils::get_args,
   err_utils::ErrStr
};

use libs::{
   collections::assets::{Assets,assets,add},
   processors::process_pools,
   reports::{report_proposes,proposal,print_table},
   types::pivots::pivot_amount
};

fn version() -> String { "1.00".to_string() }
fn app_name() -> String { "dusk".to_string() }

fn usage() -> ErrStr<()> {
   println!("Usage:

$ {} <protocol> <date>

where:
* <protocol> is the protocol-name, e.g. PIVOT
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
      let mut tokens = Assets::new();
      proposals.iter().for_each(|p| {
         let asset = pivot_amount(&proposal(p));
         add(&mut tokens, &asset);
      });
      print_table("\nAssets to pivot:", &assets(&tokens));
      Ok(())
   } else {
      usage()
   }
}

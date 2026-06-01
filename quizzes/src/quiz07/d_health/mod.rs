use std::collections::HashSet;

use book::{
   date_utils::parse_date,
   err_utils::ErrStr,
   list_utils::tail,
   string_utils::words,
   utils::get_args
};

use libs::{ processors::compute_health, reports::report_health };

fn version() -> String { "1.01".to_string() }
fn app_name() -> String { "hwaet".to_string() }
fn usage() -> ErrStr<()> {
   let app = app_name();
   println!("\n{}, version {}\n\n\t$ {} [--debug] <protocol> <date>

prints the current available assets for all pivot pools: a health-check.

where
* [-d] or [--debug] (optional) output debugging while computing health
* <protocol> is the protocol, e.g. PIVOT
* <date> to check availability, e.g.: $LE_DATE
", app,  version(), app);
   Err("Needs arguments <protocol> <date>".to_string())
}

#[cfg(not(tarpaulin_include))]
pub async fn runoff_with_args() -> ErrStr<()> {
   let args = get_args();
   if let Some(a) = args.first() {
      let ds: HashSet<String> = words("-d --debug").into_iter().collect();
      let debug = ds.contains(a);
      let args1 = if debug { tail } else { <[_]>::to_vec }(&args);
      runoff_continued(&args1, debug).await
   } else {
      usage()
   }
}

#[cfg(not(tarpaulin_include))]
async fn runoff_continued(args: &[String], debug: bool) -> ErrStr<()> {
   if let [auth, dt] = args {
      let date = parse_date(&dt)?;
      let comps = compute_health(&auth, &date, debug).await?;
      report_health(date, comps);
      Ok(())
   } else {
      usage()
   }
}

// ----- TESTS -------------------------------------------------------

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, date_utils::yesterday, utils::now };

   create_testing!("quiz07::d_health");

   run!("compute_and_report_health", {
      let yday = yesterday();
      now(runoff_continued(&words(&format!("pivot {}", yday)), true))?;
   });
}


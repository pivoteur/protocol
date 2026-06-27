use book::{
   csv_utils::as_csv,
   currency::usd::{ USD, mk_usd },
   err_utils::ErrStr,
   list_utils::tail,
   num_utils::parse_num,
   parse_utils::parse_id,
   string_utils::s,
   utils::{get_args,get_env}
};

use libs::{
   fetchers::calls::fetch_call_data,
   processors::virtuals::compute_counter_offer,
   types::{ calls::Call, pivots::Pivot }
};

fn version() -> String { s("1.04") }
fn app_name() -> String { s("offrian") }
fn usage() -> ErrStr<()> {
   let app = app_name();
   println!("
{}, version {}

Usage:

$ {} [--debug] <protocol> <ix> <part>

where:

* [-d|--debug] show debug information
* <protocol> is the protocol to make the counter-offer, e.g.: PIVOT
* <ix> is the call being countered, e.g. 1
* <vol> is the target volume (the volume we want the open pivots to have been)
  e.g.: if the open pivots were for $15074.88, say, then 3000 reduces the open 
        pivots to $3000.00
", app, version(), app);
   Err(s("offrian requires <protocol> <ix> <part> arguments"))
}

pub async fn runoff_with_args() -> ErrStr<()> {
   let args = get_args();
   if let Some(debug) = args.first() {
      let (rest, debug) = if debug == "--debug" || debug == "-d" {
         (tail(&args), true)
      } else {
         (args.clone(), false)
      };
      runoff_continuation(&rest, debug).await
   } else {
      usage()
   }
}

async fn runoff_continuation(args: &[String], debug: bool) -> ErrStr<()> {
   if let [auth, call, vol] = args {
      let root_url = get_env(&format!("{}_URL", auth.to_uppercase()))?;
      let target = mk_usd(parse_num(&vol)?);
      let ix = parse_id(&call)?;
      let call_data = fetch_call_data(&root_url, ix, debug).await?;
      let new_call = compute_counter_off((&call_data, &target, debug)?;
      report_counter_offer(&new_call, debug)
   } else {
      usage()
   }
}

fn report_counter_offer(nc: &Call, debug: bool) -> ErrStr<()> {
   if debug {
      let prop = &nc.proposed_token;
      let a = format!("swap {} {} (quote: {})", nc.pivot_amount,
                      nc.pivot_token, nc.pivot_close_price);
      let b = format!("for at least {} {} (quote: {})", nc.gain_10_percent,
                      prop, nc.proposed_close_price);
      let c  = format!("(expecting swap to {} {})", nc.proposed_amount,
                       prop);
      println!("\nOffrian:\n{a}\n{b}\n{c};\nROI {} / {} APR\n", nc.roi, nc.apr);
   }
   let output = as_csv(&[nc])?;
   println!("{output}");
   Ok(())
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;

   use paste::paste;
   use book::{ create_testing, csv_utils::as_csv, utils::now };
   use libs::processors::virtuals::test_data::sample_avax_undead_offrian;

   create_testing!("quizzes::quiz08::e_offrian", "", true);

   run!("debug_offrian",
        now(runoff_continuation(&[s("pivot"),s("1"),s("9")],true)));

   run!("offrian", now(runoff_continuation(&[s("pivot"),s("1"),s("9")],false)));

   run!("with_data_compute_offrian", {
      let (call, opens) = sample_avax_undead_offrian(".")?;
      let target = mk_usd(1700.0);
      let new_call =
         with_data_compute_offrian(&call, &opens, &target, false)?;
      println!("The new call of {target} for\n\n{}\nis\n\n{}",
               as_csv(&[call])?, as_csv(&[new_call])?);
   });
}

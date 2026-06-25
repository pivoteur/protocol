use book::{
   csv_utils::as_csv,
   currency::usd::mk_usd,
   err_utils::ErrStr,
   list_utils::tail,
   num_utils::parse_num,
   parse_utils::parse_id,
   string_utils::s,
   utils::{get_args,get_env}
};

use libs::{
   fetchers::calls::fetch_call_data,
   processors::virtuals::{
      compute_virtual_pivot_amount,
      compute_counter_offer
   },
   types::calls::Call
};

fn version() -> String { s("1.03") }
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
      let volume = mk_usd(parse_num(&vol)?);
      let ix = parse_id(&call)?;
      let (call, pivs) = fetch_call_data(&root_url, ix, debug).await?;
      let leeway =
         compute_virtual_pivot_amount(&call.pool, &pivs, &call.ids, debug);
      let leeway_vol = mk_usd(leeway * call.proposed_close_price.amount());
      if debug {
         println!("The virtual pivots provide {leeway} {} leeway ({})",
                  call.from_token, leeway_vol);
      }
      let new_call = compute_counter_offer(&call, &volume, leeway_vol, debug)?;
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
   use book::{ create_testing, csv_utils::{as_csv,enumerate_csv}, utils::now };
   use libs::fetchers::test_helpers::test_functions::marshall;

   create_testing!("quizzes::quiz08::e_offrian", "", true);

   run!("debug_offrian",
        now(runoff_continuation(&[s("pivot"),s("1"),s("9")],true)));

   run!("offrian", now(runoff_continuation(&[s("pivot"),s("1"),s("9")],false)));
}

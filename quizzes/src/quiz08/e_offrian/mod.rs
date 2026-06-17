use std::collections::HashSet;

use book::{
   csv_utils::as_csv,
   currency::usd::mk_usd,
   err_utils::ErrStr,
   list_utils::tail,
   num::percentage::mk_percentage,
   num_utils::parse_num,
   parse_utils::parse_id,
   string_utils::s,
   utils::{get_args,get_env}
};

use libs::{
   fetchers::{calls::fetch_calls,pivots::fetch_open_pivots},
   types::{ aliases::aliases, calls::Call, pools::Pool}
};

fn version() -> String { s("0.90") }
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
* <part> is the subset of the call being countered,
  e.g.: 3 is a 1/3rd counter-offer
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
   if let [auth, call, part] = args {
      let root_url = get_env(&format!("{}_URL", auth.to_uppercase()))?;
      let fract = parse_num(&part)?;
      let ix = parse_id(&call)?;
      let call = grab_call(&root_url, ix, debug).await?;
      if debug { println!("Examining call\n{call:?}"); }
      // from the call we get the committed amount and open pivots
      // from the open pivots we get the virtual amount committed;
      // that's our play or leeway.
      let pivot_amt = call.gain_10_percent / 1.1; // total pivoted
      let target_amt = pivot_amt / fract;
      let give_up = pivot_amt - target_amt;
      let leeway =
         virtual_pivots_amount(&root_url, &call.pool, &call.ids, debug).await?;
      if debug { println!("The virtual pivots provide {leeway} leeway"); }
      let gap = leeway - give_up;
      if gap < 0.0 {
         Err(format!("Unable to fracture call {ix} by {fract}; {} derth",
                     -gap))
      } else {
         let new_call = compute_offrian(&call, target_amt);
         counter_offer(&new_call, debug)
      }
   } else {
      usage()
   }
}

fn compute_offrian(call: &Call, target_amt: f32) -> Call {
   let piv_qt = call.pivot_close_price;
   let new_vol = target_amt * piv_qt.amount();
   let new_origin = new_vol / call.quote1.amount();
   let new_vol_usd = mk_usd(new_vol);
   let prop_qt = call.proposed_close_price;
   let landing_at = new_vol / prop_qt.amount();
   let at_least = new_origin * 1.1;
   let gain = landing_at - new_origin;
   let roi0 = gain / new_origin;
   let roi = mk_percentage(roi0);
   let duration = call.close_date
                      .signed_duration_since(call.opened)
                      .num_days() as f32;
   let apr = mk_percentage(roi0 * 365.0 / duration);
   let c = call.clone();
   let new_call = Call {
      virtual_amount: new_origin - call.amount1,
      val1: new_vol_usd,
      gain_10_percent: at_least,
      pivot_amount: target_amt,
      proposed_amount: landing_at,
      roi, 
      apr,
      ..c };
   new_call
}

fn counter_offer(nc: &Call, debug: bool) -> ErrStr<()> {
   if debug {
      let prop = &nc.proposed_token;
      let a = format!("swap {} {} (quote: {})", nc.pivot_amount,
                      nc.pivot_token, nc.pivot_close_price);
      let b = format!("for at least {} {} (quote: {})", nc.gain_10_percent,
                      prop, nc.proposed_close_price);
      let c  = format!("(expecting swap to {} {})", nc.proposed_amount,
                       prop);
      println!("Offrian: {a} {b} {c}; ROI {} / {} APR", nc.roi, nc.apr);
   }
   let output = as_csv(&[nc])?;
   println!("{output}");
   Ok(())
}

async fn grab_call(root_url: &str, ix: usize, debug: bool) -> ErrStr<Call> {
   let calls = fetch_calls(root_url).await?;
   if debug { println!("Fetched {} calls", calls.len()); }
   let call = calls.get(ix - 1).ok_or("No call found at index {ix}")?;
   Ok(call.clone())
}

async fn virtual_pivots_amount(root_url: &str, pool: &Pool,
                               pivs: &[usize], debug: bool) -> ErrStr<f32> {
   let a = aliases();
   let (all_pivs0, dt) = fetch_open_pivots(root_url, pool, &a).await?;
   if debug {
      println!("Fetched {} open pivots; max_date: {dt}", all_pivs0.len());
   }
   let pivs_set: HashSet<usize> = pivs.into_iter().collect();
   let mut all_pivs = all_pivs0.clone();
   // filter down to virtual pivots in the call
   all_pivs.retain(|p| p.is_virtual() && pivs_set.contains(&p.index()));
   if debug {
      println!("There are {} virtual pivotes for {pool} call", all_pivs.len());
   }
   Ok(all_pivs.iter().map(|p| p.committed()?.sz()).sum())
}

// -----------------------------------------------------------------------------
// 🚀 RUNTIME VERIFICATION
// -----------------------------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;

   use paste::paste;
   use book::{create_testing,err_utils::ErrStr};

   create_testing!("quizzes::quiz08::e_offrian");

   run!("parse_calls", {
      let raw_csv_data = "\
ix,pool,open_pivots,last_pivot_on_dt,opened,ids,close_id,close_date,from,from_blockchain,amount1,virtual,quote1,val1,gain_10_percent,pivot_token,pivot_blockchain,pivot_close_price,pivot_amount,proposed_token,proposed_blockchain,proposed_close_price,proposed_amount,roi,apr
1,BTC+USDC,10,2026-04-16,2026-04-15,27;29,8,2026-06-10,BTC,Avalanche,0,0.452206,$81812.00,$36995.88,0.4974266,USDC,Avalanche,$1.00,37005.758,BTC,Avalanche,$61419.00,0.6023795,33.21%,216.45%
2,BTC+UNDEAD,20,2026-04-09,2026-02-07,3;5;8;10;28;32;34;36;40,15,2026-06-10,UNDEAD,Avalanche,2189400,540280.56,$0.001782,$4863.69,3002648.5,BTC,Avalanche,$61419.00,0.0646658,UNDEAD,Avalanche,$0.000960,4135559.8,51.50%,152.84%";

      let parsed_records = parse_crypto_csv(raw_csv_data)?;

      for record in parsed_records {
         println!(
            "Pool: {:<11} | ROI: {:>5}% | IDs Vector: {:?}",
            record.pool, record.roi, record.ids
         );
      }
   });
}


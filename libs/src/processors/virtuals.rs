use crate::types::{
   calls::{ Call, CallData },
   pivots::opens::{ Pivot, mk_pivot },
   quotes::Quotes
};

use book::{
   csv_utils::{ CsvHeader, CsvWriter },
   currency::usd::USD,
   err_utils::ErrStr,
   string_utils::s
};

// ----- RECOMPUTING VIRTUAL PIVOTS (virtsz) -------------------------------

pub fn recompute_pivot(quotes: &Quotes, debug: bool)
      -> impl Fn(Pivot) -> ErrStr<Pivot> {
   move |p| {
      if !p.is_virtual() { Err(s("Can only recompute virtual pivots"))
      } else {
         if p.closed() { Err(s("Pivot closed; cannot recompute"))
         } else { recompute1(quotes, p, debug)
         }
      }
   }
}

fn recompute1(quotes: &Quotes, p: Pivot, debug: bool) -> ErrStr<Pivot> {
   if debug { println!("For pivot:\n{}\n{}", p.header(), p.as_csv()); }
   let mb_new_assets = p.recompute_assets(quotes)?;
   Ok(match mb_new_assets {
      Some((from, to)) => {
         let today = &quotes.date;
         let header = p.update_header(today);
         let new_piv1 = mk_pivot(header, from, to);
         if debug { println!("\tRecomputed to:\n{}", new_piv1.as_csv()); }
         new_piv1
      },
      None => {
         if debug { println!("\tNo change"); }
         p
      }
   })
}

// ----- RECOMPUTING VIRTUAL PIVOTS (offrian) -------------------------------

mod counter_offerer {
   use std::collections::HashSet;
   use book::{
      debug,
      currency::usd::{ USD, mk_usd },
      err_utils::ErrStr,
      num::percentage::mk_percentage,
      string_utils::is_are
   };

   use crate::types::{
      calls::{ Call, CallData },
      measurable::Measurable,
      pivots::opens::Pivot,
      pools::pool_names::Pool
   };

   fn compute_virtual_pivot_amount(call_data: &CallData, debug: bool) -> f32 {
      let (call, opens) = call_data;
      let mut amount = 0.0;
      let virtuals = filter_virtuals(&call.pool, &opens, &call.ids, debug);
      for v in virtuals { amount += v.sz(); }
      amount
   }

   fn filter_virtuals(pool: &Pool, all_pivots: &[Pivot],
                      opens: &[usize], debug: bool) -> Vec<Pivot> {
      debug!("filter_virtuals", debug);
      let pivs_set: HashSet<usize> = opens.iter().copied().collect();
      let mut virtuals = all_pivots.to_vec();
      // filter down to virtual pivots in the call
      virtuals.retain(|p| p.is_virtual() && pivs_set.contains(&p.index()));
      log!("There {} for {} call",
           is_are(virtuals.len(), "virtual pivot"), pool);
            
      virtuals
   }

   type AmtMin = (f32, USD);

   pub fn compute_offrian(call: &Call, target: &USD,
                          debug: bool) -> ErrStr<Call> {
      let (new_pivot_amt, floor) = compute_new_pivot_amt(call, target, debug);
      let vol = mk_usd(new_pivot_amt * call.pivot_close_price.amount());
      if vol < floor {
         let gap = mk_usd(vol.amount() - floor.amount());
         Err(format!("Unable to change call {} to {target}; {gap} derth",
                     call.ix))
      } else {
         Ok(compute_new_call(&call, new_pivot_amt))
      }
   }

   fn tv_computer(quote: &USD) -> impl Fn(f32) -> USD {
      move | amt: f32 | mk_usd(amt * quote.amount())
   }

   fn token_info(token: &str, quote: &USD) -> impl Fn(&str, f32) -> String {
      let compute_tv = tv_computer(quote);
      move | label: &str, amt: f32 | {
         format!("{label}: {amt} {token} ({})", compute_tv(amt))
      }
   }

   fn compute_new_pivot_amt(call: &Call, target: &USD, debug: bool) -> AmtMin {
      debug!("compute_new_pivot_amt", debug);
      let (new_principal, floor) = compute_new_start(call, target, debug);
      let vol = new_principal * call.quote1.amount();
      let pivot_qt = &call.pivot_close_price;
      let new_pivot = vol / pivot_qt.amount();
      let tok_inf = token_info(&call.pivot_token, pivot_qt);
      log!("New volume: {}", mk_usd(vol));
      log!("{}", tok_inf("New pivot amount", new_pivot));
      (new_pivot, floor)
   }

   fn compute_new_start(call: &Call, target: &USD, debug: bool) -> AmtMin {
      debug!("compute_new_start", debug);
      // from the call we get the committed amount and open pivots
      // from the open pivots we get the virtual amount committed;
      // that's our play or leeway.
      let principal_amt = call.gain_10_percent / 1.1; // total pivoted
      let ratio = call.val1.amount() / target.amount();
      let new_start = principal_amt / ratio;
      let tok_info = token_info(&call.from_token, &call.proposed_close_price);
      log!("{}", tok_info("principal_amt", principal_amt));
      log!("{}", tok_info("Required amount to commit to pivot", call.amount1));
      log!("ratio: {}", ratio);
      log!("{}", tok_info("new starting principal", new_start));
      (new_start, mk_usd(call.amount1 * call.proposed_close_price.amount()))
   }

   fn compute_new_call(call: &Call, target_amt: f32) -> Call {
      let piv_qt = &call.pivot_close_price;
      let new_vol = target_amt * piv_qt.amount();
      let new_origin = new_vol / call.quote1.amount();
      let new_vol_usd = mk_usd(new_vol);
      let prop_qt = &call.proposed_close_price;
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

   pub fn compute_leeway(call_data: &CallData, debug: bool) {
      debug!("compute_leeway", debug);
      let leeway = compute_virtual_pivot_amount(call_data, debug);
      let (call, _opens) = call_data;
      let leeway_info = token_info(&call.from_token, &call.quote1);
      log!("{} leeway", leeway_info("The virtual pivots provide", leeway));
   }

   #[cfg(not(tarpaulin_include))]
   #[cfg(test)]
   mod tests {
      use super::*;
      use crate::types::calls::test_data::{
         sample_undead_usdc_offrian,
         sample_call,
         target
      };
      use book::{
         csv_utils::as_csv,
         num::estimate::mk_estimate,
         string_utils::s
      };

      #[test] fn test_compute_new_pivot() -> ErrStr<()> {
         let call = sample_call(4)?;
         let (new_pivot_amt, _) = compute_new_pivot_amt(&call, &target(), true);
         let undead_est = mk_estimate(6.61e5);
         undead_est.is(new_pivot_amt)
      }

      #[test] fn test_compute_new_start() -> ErrStr<()> {
         let call = sample_call(4)?;
         let (btc, _) = compute_new_start(&call, &target(), true);
         mk_estimate(0.016).is(btc)
      }

      #[test] fn test_compute_virtual_pivot_amount_offrian() -> ErrStr<()> {
         let call_data = sample_undead_usdc_offrian("../quizzes")?;
         let (call, _) = &call_data;
         let tok = s(&call.from_token);
         let virtual_amt = compute_virtual_pivot_amount(&call_data, true);
         println!("For call:\n\n{}\nvirtual amount: {virtual_amt} {}",
                  as_csv(&[call], true)?, tok);
         assert!(virtual_amt > 0.0);
         Ok(())
      }
   }
// xxx TODO: moar testos herer
}

use counter_offerer::{ compute_leeway, compute_offrian };

pub fn compute_counter_offer(call_data: &CallData, target: &USD, debug: bool)
      -> ErrStr<Call> {
   compute_leeway(call_data, debug);
   compute_offrian(&call_data.0, target, debug)
}

// ----- TESTS -----------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use paste::paste;
   use book::create_testing;

   use crate::types::{
      assets::amounts::mk_amt,
      pivots::opens::test_data::mk_btc_usdc_piv,
      quotes::sample_data::sample_quotes_maker
   };

   create_testing!("processors::virtuals");

   run!("recompute_pivot", " (virtz)", {
      let piv = mk_btc_usdc_piv(78408.88,mk_amt(0.0,0.1),0,"virtual pivot")?;
      let quotes = sample_quotes_maker(&[("BTC", 80000.0)]);
      let _new_piv = recompute_pivot(&quotes, true)(piv)?;
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use crate::types::{
      assets::amounts::mk_amt,
      calls::test_data::{
         sample_btc_undead_offrian,
         sample_undead_usdc_offrian,
         tenk
      },
      pivots::opens::test_data::mk_btc_usdc_piv,
      quotes::sample_data::sample_quotes_maker
   };

   use book::{
      currency::usd::mk_usd,
      num::estimate::mk_estimate,
      types::values::Value
   };

   // ----- virtsz tests ------------------------------------------------------

   #[test] fn fail_recompute_non_virtual_amt_pivot() -> ErrStr<()> {
      let piv = mk_btc_usdc_piv(78408.88, mk_amt(500.0, 0.0), 0, "https://yo")?;
      let reckt =
         recompute_pivot(&sample_quotes_maker(&[("BTC", 80000.0)]), false)(piv);
      assert!(reckt.is_err());
      if let Err(x) = reckt {
         assert!(x.contains("virtual"));
         Ok(())
      } else { 
         Err(format!("reckt ({reckt:?}) succeeds (???) unfortunately."))
      }
   }

   #[test] fn fail_recompute_non_virtual_tx_pivot() -> ErrStr<()> {
      let piv = mk_btc_usdc_piv(78408.88, mk_amt(0.0, 500.0), 0, "https://yo")?;
      let reckt =
         recompute_pivot(&sample_quotes_maker(&[("BTC", 80000.0)]), false)(piv);
      assert!(reckt.is_err());
      if let Err(x) = reckt {
         assert!(x.contains("virtual"));
         Ok(())
      } else {
         Err(format!("reckt ({reckt:?}) succeeds (???) unfortunately."))
      }
   }  

   #[test] fn fail_recompute_closed_pivot() -> ErrStr<()> {
      let piv = mk_btc_usdc_piv(78408.88,mk_amt(0.0,500.0),1,"virtual pivot")?;
      let reckt =
         recompute_pivot(&sample_quotes_maker(&[("BTC",80000.0)]), false)(piv);
      assert!(reckt.is_err());
      if let Err(x) = reckt {
         assert!(x.contains("close"));
         Ok(())
      } else {
         let cls = "closed pivot recompute";
         Err(format!("{cls} {reckt:?} succeeds (???) unfortunately."))
      }
   }

   #[test] fn test_no_recompute_virtual_pivot_ok() -> ErrStr<()> {
      let piv = mk_btc_usdc_piv(78408.88,mk_amt(0.0, 0.1),0,"virtual_pivot")?;
      assert!(!piv.is_updated());
      let neiner =
         recompute_pivot(&sample_quotes_maker(&[("BTC",65000.0)]), false)(piv);
      assert!(neiner.is_ok());
      assert!(!neiner.unwrap().is_updated());
      Ok(())
   }

   // ----- offrian tests -----------------------------------------------------

   #[test] fn fail_compute_counter_offer() -> ErrStr<()> {
      let call_data = sample_undead_usdc_offrian("../quizzes")?;
      let truthiness = compute_counter_offer(&call_data, &mk_usd(1000.0), true);
      assert!(truthiness.is_err());
      Ok(())
   }

   #[test] fn test_compute_counter_offer_ok() -> ErrStr<()> {
      let call_data = sample_undead_usdc_offrian("../quizzes")?;
      let truthiness =
         compute_counter_offer(&call_data, &mk_usd(8e4), true);
      assert!(truthiness.is_ok(), "Err is {truthiness:?}");
      Ok(())
   }

   #[test] fn test_compute_offrian() -> ErrStr<()> {
      let call_data = sample_undead_usdc_offrian("../quizzes")?;
      compute_leeway(&call_data, true);
      let (call, _opens) = call_data;
      let new_call = compute_offrian(&call, &tenk(), true)?;
      let roi_est = mk_estimate(0.46);
      roi_est.is(new_call.roi.value())?;
      let apr_est = mk_estimate(6.23);
      apr_est.is(new_call.apr.value())?;
      let undead_est = mk_estimate(1.55e7);
      undead_est.is(new_call.amount1)
   }

   #[test] fn test_compute_counter_offer_positive_virtual_amount()
         -> ErrStr<()> {
      let call_data = sample_btc_undead_offrian("../quizzes")?;
      let target = mk_usd(1700.00);
      let call = compute_counter_offer(&call_data, &target, true)?;
      let virt = call.virtual_amount;
      assert!(virt > 0.0, "Virtual amount ({virt}) cannot be negative");
      Ok(())
   }
}


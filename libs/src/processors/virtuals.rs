use std::collections::HashSet;

use crate::types::{
   calls::Call,
   measurable::Measurable,
   pivots::{ Pivot, mk_pivot },
   pools::Pool,
   quotes::Quotes
};

use book::{
   csv_utils::{ CsvHeader, CsvWriter },
   currency::usd::{ USD, mk_usd, no_monay },
   err_utils::ErrStr,
   num::percentage::mk_percentage,
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

pub fn compute_virtual_pivot_amount(pool: &Pool, all_pivots: &[Pivot],
                                opens: &[usize], debug: bool) -> f32 {
   let mut amount = 0.0;
   let virtuals = filter_virtuals(pool, all_pivots, opens, debug);
   for v in virtuals { amount += v.sz(); }
   amount
}

fn filter_virtuals(pool: &Pool, all_pivots: &[Pivot],
                   opens: &[usize], debug: bool) -> Vec<Pivot> {
   let pivs_set: HashSet<usize> = opens.iter().copied().collect();
   let mut virtuals = all_pivots.to_vec();
   // filter down to virtual pivots in the call
   virtuals.retain(|p| p.is_virtual() && pivs_set.contains(&p.index()));
   if debug {
      println!("There are {} virtual pivots for {pool} call", virtuals.len());
   }
   virtuals
}

pub fn compute_counter_offer(call: &Call, target: &USD,
                             leeway_vol: USD, debug: bool) -> ErrStr<Call> {
   let new_pivot_amt = compute_new_pivot_amt(call, target, debug);
   let give_up = call.pivot_amount - new_pivot_amt;
   let give_up_vol = mk_usd(give_up * call.pivot_close_price.amount());
   let gap_vol = leeway_vol - give_up_vol;
   if gap_vol < no_monay() {
      Err(format!("Unable to change call {} to {target}; {} derth",
                  call.ix, gap_vol))
   } else {
      Ok(compute_offrian(&call, new_pivot_amt))
   }
}

fn compute_new_pivot_amt(call: &Call, target: &USD, debug: bool) -> f32 {
   let new_principal = compute_new_start(call, target, debug);
   let vol = new_principal * call.quote1.amount();
   if debug { println!("New volume: {}", mk_usd(vol)); }
   let new_pivot = vol / call.pivot_close_price.amount();
   if debug { println!("New pivot amount: {new_pivot} {}", call.pivot_token); }
   new_pivot
}

fn compute_new_start(call: &Call, target: &USD, debug: bool) -> f32 {
   // from the call we get the committed amount and open pivots
   // from the open pivots we get the virtual amount committed;
   // that's our play or leeway.
   let principal_amt = call.gain_10_percent / 1.1; // total pivoted
   if debug { println!("principal_amt: {principal_amt} {}", call.from_token); }
   let new_start = target.amount() / call.quote1.amount();
   if debug {
      println!("new_start: {new_start} {} ({target})", call.from_token);
   }
   new_start
}

fn compute_offrian(call: &Call, target_amt: f32) -> Call {
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


// ----- TESTS -----------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod test_data {
   use super::*;
   use crate::types::calls::parse_calls;

   pub fn sample_call() -> ErrStr<Call> {
      let raw_csv_data = "\
ix,pool,open_pivots,last_pivot_on_dt,opened,ids,close_id,close_date,from,from_blockchain,amount1,virtual,quote1,val1,gain_10_percent,pivot_token,pivot_blockchain,pivot_close_price,pivot_amount,proposed_token,proposed_blockchain,proposed_close_price,proposed_amount,roi,apr
1,BTC+USDC,10,2026-04-16,2026-04-15,27;29,8,2026-06-10,BTC,Avalanche,0,0.452206,$81812.00,$36995.88,0.4974266,USDC,Avalanche,$1.00,37005.758,BTC,Avalanche,$61419.00,0.6023795,33.21%,216.45%
2,BTC+UNDEAD,20,2026-04-09,2026-02-07,3;5;8;10;28;32;34;36;40,15,2026-06-10,UNDEAD,Avalanche,2189400,540280.56,$0.001782,$4863.69,3002648.5,BTC,Avalanche,$61419.00,0.0646658,UNDEAD,Avalanche,$0.000960,4135559.8,51.50%,152.84%";
      let calls = parse_calls(raw_csv_data)?;
      Ok(calls.first().unwrap().clone())
   }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, csv_utils::as_csv, utils::now };

   use crate::{
      fetchers::{
         calls::fetch_call_data,
         test_helpers::test_functions::marshall
      },
      types::{
         assets::amounts::mk_amt,
         pivots::sample_pivots::mk_btc_usdc_piv,
         quotes::sample_data::sample_quotes_maker
      }
   };

   create_testing!("processors::virtuals");

   run!("recompute_pivot", " (virtz)", {
      let piv = mk_btc_usdc_piv(78408.88,mk_amt(0.0,0.1),0,"virtual pivot")?;
      let quotes = sample_quotes_maker(&[("BTC", 80000.0)]);
      let _new_piv = recompute_pivot(&quotes, true)(piv)?;
   });

   run!("compute_virtual_pivot_amount", " (offrian)", {
      let (root_url, _) = marshall()?;
      let (call, pivs) = now(fetch_call_data(&root_url, 1, true))?;
      let pool = &call.pool;
      let opens = &call.ids;
      let tok = s(&call.from_token);
      let virtual_amt = compute_virtual_pivot_amount(pool, &pivs, opens, true);
      println!("For call:\n\n{}\nvirtual amount: {virtual_amt} {}",
               as_csv(&[call])?, tok);
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use super::test_data::sample_call;
   use crate::{
      types::{
         assets::amounts::mk_amt,
         pivots::sample_pivots::mk_btc_usdc_piv,
         quotes::sample_data::sample_quotes_maker
      }
   };

   use book::{ num::estimate::mk_estimate, types::values::Value };

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

   fn target() -> USD { mk_usd(2500.0) }

   #[test] fn fail_compute_counter_offer() -> ErrStr<()> {
      let call = sample_call()?;
      let truthiness =
         compute_counter_offer(&call, &target(), mk_usd(100.0), true);
      assert!(truthiness.is_err());
      Ok(())
   }

   #[test] fn test_compute_counter_offer_ok() -> ErrStr<()> {
      let call = sample_call()?;
      let truthiness =
         compute_counter_offer(&call, &target(), mk_usd(35000.0), true);
      assert!(truthiness.is_ok(), "Err is {truthiness:?}");
      Ok(())
   }

   #[test] fn test_compute_new_pivot() -> ErrStr<()> {
      let call = sample_call()?;
      let new_pivot = compute_new_pivot_amt(&call, &target(), true);
      assert_eq!(2500.0, new_pivot);
            // only works on USDC pools which this call happens to be on.
      Ok(())
   }

   #[test] fn test_compute_new_start() -> ErrStr<()> {
      let call = sample_call()?;
      let btc = compute_new_start(&call, &target(), true);
      mk_estimate(0.03).is(btc)
   }

   #[test] fn test_compute_offrian() -> ErrStr<()> {
      let call = sample_call()?;
      let new_call = compute_offrian(&call, 1000.0);
      let roi_est = mk_estimate(0.33);
      roi_est.is(new_call.roi.value())?;
      let apr_est = mk_estimate(2.16);
      apr_est.is(new_call.apr.value())?;
      let btc = new_call.amount1;
      assert_eq!(0.0, btc, "BTC: principal asset (actual, not virtual)");
      let btc = new_call.virtual_amount;
      let btc_est = mk_estimate(0.45 / 37.0);
      btc_est.is(btc)
   }
}

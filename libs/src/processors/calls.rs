use book::{
   currency::usd::mk_usd,
   num::percentage::mk_percentage,
   types::values::Value
};

use crate::types::{ calls::Call, pivots::closes::{mk_close_pivot, ClosePivot} };

pub fn transform_to_close(c: &Call, tx_id: &str, actual: f32) -> ClosePivot {
   let from = c.amount1 + c.virtual_amount;
   let gain = actual - from;
   let roi = mk_percentage(gain / from);
   let close_dt = &c.close_date;
   let opn_dt = &c.opened;
   let days = (close_dt.clone() - opn_dt.clone()).num_days() as f32;
   let apr_raw = roi.value() * 365.0 / days;
   let apr = mk_percentage(apr_raw);
   let out_quote = &c.proposed_close_price;
   let in_quote = &c.pivot_close_price;
   let vol = mk_usd(c.pivot_amount * in_quote.amount());
   let gain_usd = mk_usd(gain * out_quote.amount());
   mk_close_pivot(close_dt, Some(&c.opened), &c.ids, c.close_id, tx_id,
                  &c.pivot_token, in_quote, &c.proposed_token, out_quote,
                  c.pivot_amount, &vol, c.gain_10_percent, actual, gain,
                  &gain_usd, &roi, &apr)
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use paste::paste;
   use super::*;
   use book::{ create_testing, csv_utils::as_csv, err_utils::ErrStr };
   use crate::types::calls::test_data::sample_call;

   create_testing!("processors::calls");

   run!("transform_to_close", {
      let c = sample_call(1)?;
      let close_piv = transform_to_close(&c, "asdf", 0.62);
      println!("The close pivot from:\n{}\nis\n{}",
               as_csv(&[c], true)?, as_csv(&[close_piv], true)?);
   });
}

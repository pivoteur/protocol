/// utilities around pivot-computations or with sets of pivots

use chrono::{ NaiveDate, Days };

use book::{ err_utils::ErrStr, string_utils::s };

use crate::types::measurable::{ Measurable, size };

/// Computes both 
/// * the weighted_day, which is the opening date of several open pivots
///   (broken apart into `dates` and `principal`); and,
/// * the duration from that `weighted_day` to now, the `close_date`.

pub fn weighted_days<M: Measurable>(dates: &[NaiveDate], principal: &[M],
                     close_date: &NaiveDate) -> ErrStr<(f32, NaiveDate)> {
   let sunshine = durations_on_dates(dates)?;
   let (start_date, days) = &sunshine;
   let weights: Vec<f32> =
      days.iter()
          .zip(principal.iter().map(Measurable::sz))
          .map(|(&a, b)| a * b)
          .collect();
   let wt: f32 = weights.iter().sum();
   let wt_days = wt / size(principal);
   let ave_dt = *start_date + Days::new((wt_days - 1.0) as u64);
   let dur = *close_date - ave_dt;
   let duration = dur.num_days() as f32;
   Ok((duration, ave_dt))
}

type Rays<T> = (T, Vec<f32>);

fn durations_on_dates(dates: &[NaiveDate]) -> ErrStr<Rays<NaiveDate>> {
   if let Some(start_date) = dates.iter().min().cloned() {
      Ok((start_date, dates.iter()
             .map(|&d| ((d-start_date).num_days() + 1) as f32)
             .collect()))
   } else {
      Err(s("No start date in empty list."))
   }
}

// ------ TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use book::{
      date_utils::{ parse_date, today },
      list_utils::filter_map_or
   };
   use crate::types::assets::assets::sample_assets::btc_asset;

   fn dates() -> ErrStr<Vec<NaiveDate>> {
      let dates_strs = vec!["2025-05-06", "1976-07-04", "2026-07-14"];
      filter_map_or(parse_date, dates_strs)
   }

   #[test] fn fail_no_durations_no_dates() {
      let ans = durations_on_dates(&[]);
      assert!(ans.is_err());
   }

   #[test] fn test_durations_on_dates() -> ErrStr<()> {
      let (start_date, durs) = durations_on_dates(&dates()?)?;
      assert!(start_date < today(),
              "start_date {start_date} is after {}", today());
      assert!(durs.iter().all(|d| *d > 0.0),
              "There are some negative durations in {durs:?}");
      Ok(())
   }

   #[test] fn fail_weighted_days() {
      let ans = weighted_days(&[], &[()], &today());
      assert!(ans.is_err());
   }

   #[test] fn test_weighted_days() -> ErrStr<()> {
      let assets =
         vec![btc_asset(1.0, 118000.0),
              btc_asset(0.5, 75237.0), btc_asset(0.75, 63234.1)];
      let (longer, opening_date) = weighted_days(&dates()?, &assets, &today())?;
      assert!(opening_date < today(),
              "opening_date {opening_date} is not before today {}", today());
      assert!(longer > 0.0, "days_from_today {longer} is not positive");
      Ok(())
   }
}

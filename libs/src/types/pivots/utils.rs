/// utilities around pivot-computations or with sets of pivots

use chrono::NaiveDate;

use book::{ err_utils::ErrStr, string_utils::s };

use super::opens::Pivot;

type Rays<T> = (T, Vec<f32>);

pub fn durations(opens: &[Pivot]) -> ErrStr<Rays<NaiveDate>> {
   let dates: Vec<NaiveDate> = opens.into_iter().map(Pivot::opened).collect();
   durations_on_dates(&dates)
}

pub fn durations_on_dates(dates: &[NaiveDate]) -> ErrStr<Rays<NaiveDate>> {
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

   #[test] fn fail_no_durations_no_dates() {
      let ans = durations_on_dates(&[]);
      assert!(ans.is_err());
   }

   #[test] fn test_durations_on_dates() -> ErrStr<()> {
      let dates_strs = vec!["2025-05-06", "1976-07-04", "2026-07-14"];
      let dates = filter_map_or(parse_date, dates_strs)?;
      let (start_date, durs) = durations_on_dates(&dates)?;
      assert!(start_date < today(),
              "start_date {start_date} is after {}", today());
      assert!(durs.iter().all(|d| *d > 0.0),
              "There are some negative durations in {durs:?}");
      Ok(())
   }
}

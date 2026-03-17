use std::fmt::Display;

use chrono::NaiveDate;

use book::{ csv_utils::{CsvHeader,CsvWriter}, err_utils::ErrStr };

use crate::types::{ headers::Header, util::Id };

#[derive(Debug, Clone)]
pub struct AggregateHeader {
   opened: Vec<NaiveDate>,
   ids: Vec<Id>,
}

// ----- CONSTRUCTOR ---------------------------------------------------------

pub fn add_header_info(v: &Vec<Header>) -> AggregateHeader {
   let mut opened = Vec::new();
   let mut ids = Vec::new();
   for h in v {
      opened.push(h.opened());
      ids.push(h.ix());
   }
   AggregateHeader { opened, ids }
}

impl AggregateHeader {
   pub fn durations(&self) -> ErrStr<(NaiveDate,Vec<f32>)> {
      if let Some(start_date) = self.opened.first().cloned() {
         Ok((start_date, self.opened.iter()
                .map(|&d| ((d-start_date).num_days() + 1) as f32)
                .collect()))
      } else {
         Err("No start date for proposal".to_string())
      }
   }
}

impl CsvHeader for AggregateHeader {
   fn header(&self) -> String { "opened,ids".to_string() }
}
impl CsvWriter for AggregateHeader {
   fn ncols(&self) -> usize { 2 }
   fn as_csv(&self) -> String {
      fn list2str<T:Display>(v: &Vec<T>) -> String {
         v.iter().map(|s| format!("{s}")).collect::<Vec<_>>().join(";")
      }
      format!("{}", list2str(&self.ids))
   }
}

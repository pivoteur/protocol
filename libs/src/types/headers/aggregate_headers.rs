use std::fmt::Display;

use chrono::NaiveDate;

use book::csv_utils::{CsvHeader,CsvWriter};

use crate::types::util::Id;

use super::headers::Header;

#[derive(Debug, Clone)]
pub struct AggregateHeader {
   opened: Vec<NaiveDate>,
   ids: Vec<Id>,
}

fn add_header_info(v: &Vec<Header>) -> AggregateHeader {
   let mut opened = Vec::new();
   let mut ids = Vec::new();
   for h in v {
      opened.push(h.opened.clone());
      ids.push(h.id);
   }
   AggregateHeader { opened, ids }
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

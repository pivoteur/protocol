use std::collections::HashMap;

use chrono::NaiveDate;

use book::{
   csv_utils::{CsvHeader,CsvWriter},
   date_utils::parse_date,
   err_utils::ErrStr,
   parse_utils::parse_id
};

use crate::types::util::Id;

// ----- HEADER

#[derive(Debug, Clone)]
pub struct Header {
   opened: NaiveDate,
   id: Id,
   close: Id,
   tx_id: String,
   updated: Option<NaiveDate>
}

impl Header {
   pub fn no_url(&self) -> bool { !self.tx_id.starts_with("https://") }
   pub fn opened(&self) -> NaiveDate { self.opened.clone() }
   pub fn closed(&self) -> bool { self.close > 0 }
   pub fn ix(&self) -> usize { self.id }
   pub fn is_updated(&self) -> bool {
      self.updated.and_then(|d| Some(d > self.opened)).unwrap_or(false)
   }
   pub fn update_to(&self, today: NaiveDate) -> Header {
      Header { updated: Some(today), ..self.clone() }
   }

}

impl CsvWriter for Header {
   fn ncols(&self) -> usize { 5 }
   fn as_csv(&self) -> String {
      fn write_updated(h: &Header) -> String {
         match h.updated {
            None => "n/a".to_string(),
            Some(x) => format!("{x}")
         }
      }
      format!("{},{},{},{},{}", self.opened,self.id,self.close,self.tx_id,
              write_updated(&self))
   }
}
impl CsvHeader for Header {
   fn header(&self) -> String { "opened,open,close,tx_id,updated".to_string() }
}

pub fn mk_hdr(opend: &str, id: Id, close: Id, tx_id: String,
          updated: Option<NaiveDate>) -> ErrStr<Header> {
   let opened = parse_date(opend)?;
   Ok(Header { opened, id, close, tx_id, updated })
}

pub fn parse_header(hdrs: &HashMap<String, usize>, row: &Vec<String>)
      -> ErrStr<Header> {
   let dt = &row[hdrs["opened"]];
   let opn = hdrs.get("open")
                 .or(hdrs.get("pivot"))
                 .ok_or("Can't find id for pivot".to_string())?;
   let id = parse_id(&row[*opn])?;
   let cls = hdrs.get("close")
                 .ok_or("Can't find close (id) for pivot".to_string())?;
   let closed = parse_id(&row[*cls])?;
   let updated = hdrs.get("updated").and_then(|ix| parse_date(&row[*ix]).ok());
   mk_hdr(dt, id, closed, row[hdrs["tx_id"]].clone(),updated)
}

pub fn next_close_id(hdrs: &Vec<Header>) -> Id {
   hdrs.iter().map(|h| h.close).max().unwrap_or(0) + 1
}


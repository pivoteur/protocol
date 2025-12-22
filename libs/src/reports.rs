use chrono::NaiveDate;

use book::csv_utils::CsvWriter;

use crate::types::{
   pivots::Propose,
   measurable::Measurable,
   util::{CsvHeader,Pool}
};

pub fn header(prim: &str, piv: &str) -> String {
   format!("{}+{}", prim.to_uppercase(), piv.to_uppercase())
}

fn print_row<T:CsvWriter + CsvHeader>(printer: impl Fn(&String) -> (),
                                      first_time: &mut bool, row: &T) {
   if *first_time {
      printer(&row.header());
      *first_time = false;
   }
   printer(&row.as_csv());
}

#[derive(Debug, Clone)]
pub struct Proposal {
   pool: String,
   opens: usize,
   max_date: NaiveDate,
   proposal: Propose
}

pub fn mk_proposal(pool: &Pool, max_date: NaiveDate, opens: usize, p: Propose)
   -> Proposal {
   let (prim, piv) = pool;
   Proposal { pool: header(prim, piv), opens, max_date, proposal: p }
}

pub fn proposal(p: &Proposal) -> Propose {
   p.proposal.clone()
}

impl CsvHeader for Proposal {
   fn header(&self) -> String {
      format!("pool,open_pivots,last_pivot_on_dt,{}", self.proposal.header())
   }
}

impl CsvWriter for Proposal {
   fn ncols(&self) -> usize { 3 + self.proposal.ncols() }
   fn as_csv(&self) -> String {
      format!("{},{},{},{}", self.pool, self.opens, self.max_date,
              self.proposal.as_csv())
   }
}

impl Measurable for Proposal {
   fn sz(&self) -> f32 { proposal(&self).sz() }
   fn aug(&self) -> f32 { proposal(&self).aug() }
}

pub fn print_table<T: CsvHeader + CsvWriter>(header: &str, v: &Vec<T>) {
   fn printer(s: &String) { println!("{s}"); }
   let mut first_time = true;
   println!("{header}\n");
   for row in v {
      print_row(printer, &mut first_time, row);
   }
}

pub fn report_proposes(proposes: &Vec<Proposal>, no_closers: &Vec<Pool>) {
   print_table("", proposes);
   if !no_closers.is_empty() { 
      println!("\nPivot pools with no closes:\n");
      for (prim, piv) in no_closers {
         println!("* {}", header(prim, piv));
      }
   }
}


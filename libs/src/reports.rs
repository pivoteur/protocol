use std::cmp::Reverse;

use chrono::NaiveDate;

use book::{
   csv_utils::{CsvHeader, CsvWriter, mk_blank},
   currency::usd::USD,
   types::indexed::mk_idx_offset
};

use super::{
   tables::{c2t, csv2tsv},
   types::{
      measurable::{Measurable,tvl},
      proposals::proposes::Propose,
      util::{Pool,pool_name}
   }
};

pub fn total_line(skip: usize, header: &str, total: &USD) {
   let pre = mk_blank(skip);
   println!("\n{}{header}:,{total}", pre.as_csv());
}

fn print_row<T: CsvWriter + CsvHeader>(printer: impl Fn(&String) -> (),
                                       first_time: &mut bool, row: &T) {
   if *first_time {
      printer(&row.header());
      *first_time = false;
   }
   printer(&row.as_csv());
}

fn print_tsv_row<T: CsvWriter + CsvHeader>(printer: impl Fn(&String) -> (),
                                           first_time: &mut bool, row: &T) {
   if *first_time {
      printer(&c2t(&row.header()));
      *first_time = false;
   }
   printer(&csv2tsv(row));
}

#[derive(Debug, Clone)]
pub struct Proposal {
   pool: String,
   opens: usize,
   max_date: NaiveDate,
   proposal: Propose,
}

pub fn mk_proposal(pool: &Pool, max_date: NaiveDate, opens: usize, p: Propose)
       -> Proposal {
   Proposal { pool: pool_name(&pool), opens, max_date, proposal: p }
}

pub fn proposal(p: &Proposal) -> Propose { p.proposal.clone() }

impl CsvHeader for Proposal {
   fn header(&self) -> String {
      format!("pool,open_pivots,last_pivot_on_dt,{}",self.proposal.header())
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

pub fn print_table<T: CsvHeader + CsvWriter>(header: &str, v: &[T]) {
   print_table_d(header, v, true);
}

fn printer(s: &String) { println!("{s}"); }

pub fn print_table_d<T: CsvHeader + CsvWriter>(header: &str, v: &[T],
                                               debug: bool) {
   let mut first_time = true;
   if debug { println!("\n{header}\n"); }
   for row in v { print_row(printer, &mut first_time, row); }
}

pub fn print_tsv_table_d<T: CsvHeader + CsvWriter>(header: &str, v: &[T],
                                                   debug: bool) {
   let mut first_time = true;
   if debug { println!("\n{}\n", c2t(header)); }
   for row in v { print_tsv_row(printer, &mut first_time, row); }
}

pub fn report_proposes(proposes: Vec<Proposal>, no_closes: &[Pool], min: bool) {
   let (pools, len) = if proposes.is_empty() {
      if !min { println!("\nNo close pivots."); }
      ("analyzed", 12)
   } else {
      let mut ps = proposes;
      ps.sort_by_key(|pool| Reverse(tvl(pool)));
      let ix_props: Vec<_> =
         ps.iter().enumerate().map(mk_idx_offset(1)).collect();
      print_table_d("Close Pivot Calls", &ix_props, !min);
      ("with no closes", ps.first().unwrap().ncols())
   };
   compact(&format!("Pivot pools {pools}"), "No pools without close calls",
           no_closes, len, min);
}

pub fn compact(hdr: &str, nada: &str, nocloses: &[Pool], ln: usize, min: bool) {
   if nocloses.is_empty() {
      if !min { println!("\n{nada}\n"); }
   } else {
      print_compact(hdr, nocloses, ln);
   }
}

fn print_compact(hdr: &str, no_closers: &[Pool], ncols: usize) {
   let len = no_closers.len();
   let nrows: usize = len * 2 / ncols + 1; // each entry takes two columns
   let entries_per_row: usize = len / nrows;
   println!("\n{hdr}\n");
   no_closers.chunks(entries_per_row).for_each(|val| {
      println!("{}", val.iter().map(pool_name).collect::<Vec<_>>().join(", ,"));
   });
}

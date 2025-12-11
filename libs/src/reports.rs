use chrono::NaiveDate;

use book::csv_utils::CsvWriter;

use crate::types::util::CsvHeader;

pub fn preamble(first_time: bool, prim: &str, piv: &str, len: usize,
                max_date: &NaiveDate, date: &NaiveDate) {
   let hdr = header(prim, piv);
   let pool = format!("{hdr} pivot pool");

   println!("{hdr}\n");

   if first_time {
      println!("There are {len} open pivots for the {pool}.");
      println!("The last entry is on {max_date}.");
      println!("Recommendations are made for token quotes on {date}.\n");
   }
}

pub fn header(prim: &str, piv: &str) -> String {
   format!("{}+{}", prim.to_uppercase(), piv.to_uppercase())
}

pub fn print_table<T:CsvWriter + CsvHeader>(printer: impl Fn(&String) -> (),
                                            first_time: &mut bool, empty: &str,
                                            rows: &Vec<T>) {
   println!("");
   if rows.is_empty() {
      println!("{empty}\n");
   } else {
      for o in rows {
         if *first_time {
            printer(&o.header());
            *first_time = false;
         }
         printer(&o.as_csv());
      }
      println!("");
   }
}


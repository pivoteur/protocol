use chrono::NaiveDate;

use book::csv_utils::CsvWriter;

use crate::types::util::CsvHeader;

pub fn preamble(prim: &str, piv: &str, len: usize,
                max_date: &NaiveDate, date: &NaiveDate) {
   let cap_prim = prim.to_uppercase();
   let cap_piv = piv.to_uppercase();
   let header = format!("{cap_prim}+{cap_piv}");
   let pool = format!("{header} pivot pool");

   println!("{header}\n");
   println!("There are {len} open pivots for the {pool}.");
   println!("The last entry is on {max_date}.");
   println!("Recommendations are made for token quotes on {date}.\n");
}

pub fn print_table<T:CsvWriter + CsvHeader>(printer: impl Fn(&String) -> (),
                                            empty: &str, rows: &Vec<T>) {
   println!("");
   if rows.is_empty() {
      println!("{empty}\n");
   } else {
      let mut print_header = true;
      for o in rows {
         if print_header {
            printer(&o.header());
            print_header = false;
         }
         printer(&o.as_csv());
      }
      println!("");
   }
}


use chrono::NaiveDate;

use book::csv_utils::CsvWriter;

use crate::types::util::CsvHeader;

pub fn header(prim: &str, piv: &str) -> String {
   format!("{}+{}", prim.to_uppercase(), piv.to_uppercase())
}

pub fn print_table<T:CsvWriter + CsvHeader>(printer: impl Fn(&String) -> (),
                                            first_time: &mut bool, 
                                            prim: &str, piv: &str, len: usize, 
                                            max_date: &NaiveDate,
                                            empty: &str, rows: &Vec<T>) {
                                           
   let hdr = header(prim, piv);
   if rows.is_empty() {
      println!("{},{empty}",hdr);
   } else {
      for o in rows {
         if *first_time {
            let header = 
               format!("pool,#open_pivot,last_pivot_on_dt,{}",o.header());
            printer(&header);
            *first_time = false;
         }
         printer(&format!("{hdr},{len},{max_date},{}",o.as_csv()));
      }
   }
}


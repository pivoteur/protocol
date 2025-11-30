use chrono::NaiveDate;

pub fn report(prim: &str, piv: &str, len: usize,
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

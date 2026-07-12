use std::{ collections::HashMap, fs::File, io };
use csv::Reader;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use book::{
   currency::usd::USD,
   err_utils::ErrStr
};

use crate::types::pivots::closes::{ ClosePivot, transform };

mod converter {
   use crate::types::pivots::closes::OldClosePivotRow;
   /// We convert old-style close pivots to the one with the 10% gain column

   // these fields are the only information we care about from the open 
   // pivot pool.  Maps only what we need from the open pivots table
   #[derive(Debug, Deserialize)]
   struct OpenPivotRow {
       #[serde(alias = "open")]
       pivot: String,
       close: String,
       #[serde(alias = "10% gain")]
       gain_10_percent: f32
   }

   /// With the above 2 structs, we can construct the new close pivot

   fn gain_10_percent_for_close(open_map: &Opens, close: &OldClosePivotRow)
         -> ErrStr<f32> {
      // Extract the 10% gain matching against the compound key mapping
      let mut gain_10 = 0.0;
      for target_pivot in close.pivot.split(|c| c == ',' || c == ';') {
         let lookup_key = (s(target_pivot), close.close.clone());

         gain_10 += open_map
               .get(&lookup_key)
               .ok_or(format!("No gain 10% for open pivot {lookup_key:?}"))?;
      }
      Ok(gain_10)
   }
}

use converter::{ OpenPivotRow, gain_10_percent_for_close };

pub type Opens = HashMap<(String, String), f32>;

pub fn process_open_pivots<R: io::Read>(opens: R) -> ErrStr<Opens> {
   // 1. Parse Open Pivots into a lookup map using (pivot, close)
   //    as a compound key
   let mut open_rdr = csv::ReaderBuilder::new()
       .delimiter(b'\t')
       .from_reader(opens);

   let mut open_map = HashMap::new();
   let mut ix = 0;
   for result in open_rdr.deserialize() {
       ix += 1;
       let row: OpenPivotRow = err_or(result,
            &format!("Cannot convert open pivot row, ix: {ix}"))?;
       let key = (row.pivot.clone(), row.close.clone());
       open_map.insert(key, row.gain_10_percent);
   }
   Ok(open_map)
}

pub type Closes<R> = Reader<R>;

pub fn process_old_close_pivots<R: io::Read>(closes: R)
       -> ErrStr<Closes<R>> {
   Ok(csv::ReaderBuilder::new().delimiter(b'\t').from_reader(closes))
}

pub fn new_close_pivots<R: io::Read>(opens: &Opens, closes: &mut Closes<R>)
      -> ErrStr<Vec<NewClosePivotRow>> {
   let mut new_closes = Vec::new();

   let mut ix = 0;
   // 3. Process records and write new format
   for result in closes.deserialize() {
      ix += 1;
      let old_row: OldClosePivotRow = err_or(result,
           &format!("Cannot convert old close pivot row, ix: {ix}"))?;
      let gain_10 = gain_10_percent_for_close(&opens, &old_row)?;
      let new_row = transform(&old_row, gain_10);
      new_closes.push(new_row);
   }
   Ok(new_closes)
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod sample_data {
   use book::string_utils::s;
   pub fn sample_open_pivots() -> String {
      s("\
open\tclose\ttx_id\t10% gain
1\t0\txx1\t12.0\n2\t1\txx2\t22.6\n4\t2\tyyy\t1939.31\n13\t1\txx3\t26.9\n")
   }

   pub fn sample_old_close_pivots() -> String {
       s("\
date\tpivot\tclose\ttx_id\tfrom\tfrom quote\tto\tto quote\ttrade\tvol\tnew to-actual\tgain\tgain, total $\tROI\tAPR
2025-08-03\t2,13\t1\thttp...\tUNDEAD\t$0.002538\tBTC\t$113,828\t483,094\t$1,226.09\t0.009426\t0.000986\t$112.27\t11.69%\t355.46%
2025-08-04\t4\t2\thttp...\tUNDEAD\t$0.008462\tBTC\t$114,529\t229179\t$1,939.31\t0.015258\t0.011058\t$1,266.48\t263.29%\t5652.99%
")
   }
}


use std::{ collections::HashMap, io };
use csv::Reader;

use book::err_utils::{ ErrStr, err_or };

use crate::types::pivots::closes::{ transform, OldClosePivotRow, ClosePivot };

mod converter {
   use serde::Deserialize;
   use crate::types::pivots::closes::OldClosePivotRow;
   use super::Opens;

   use book::err_utils::ErrStr;

   /// We convert old-style close pivots to the one with the 10% gain column

   // these fields are the only information we care about from the open 
   // pivot pool.  Maps only what we need from the open pivots table
   #[derive(Debug, Deserialize)]
   pub struct OpenPivotRow {
       #[serde(alias = "open")]
       pivot: usize,
       #[serde(alias = "10% gain")]
       gain_10_percent: f32
   }

   impl OpenPivotRow {
      pub fn key(&self) -> usize { self.pivot }
      pub fn gain(&self) -> f32 { self.gain_10_percent }
   }

   /// With the open pivot information and the (old-style) close pivot,
   /// we can construct the new close pivot

   pub fn gain_10_percent_for_close(open_map: &Opens, close: &OldClosePivotRow)
         -> ErrStr<f32> {
      // Extract the 10% gain matching against the compound key mapping
      let mut gain_10 = 0.0;
      for target_pivot in close.open_pivots_ix() {
         let lookup_key = target_pivot;

         gain_10 += open_map
               .get(&lookup_key)
               .ok_or(format!("No gain 10% for open pivot {lookup_key:?}"))?;
      }
      Ok(gain_10)
   }
}

use converter::{ OpenPivotRow, gain_10_percent_for_close };

pub type Opens = HashMap<usize, f32>;

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
       open_map.insert(row.key(), row.gain());
   }
   Ok(open_map)
}

pub type Closes<R> = Reader<R>;

pub fn process_old_close_pivots<R: io::Read>(closes: R)
       -> ErrStr<Closes<R>> {
   Ok(csv::ReaderBuilder::new().delimiter(b'\t').from_reader(closes))
}

pub fn new_close_pivots<R: io::Read>(opens: &Opens, closes: &mut Closes<R>)
      -> ErrStr<Vec<ClosePivot>> {
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

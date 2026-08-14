use std::io;
use csv::Reader;

use book::err_utils::{ ErrStr, err_or };

use crate::types::{
   measurable::size,
   pivots::{
      closes::{ transform, OldClosePivotRow, ClosePivot },
      opens::Pivot,
      utils::weighted_date
   }
};
use super::opens::filter_pivots;

pub type Closes<R> = Reader<R>;

pub fn process_old_close_pivots<R: io::Read>(closes: R)
       -> ErrStr<Closes<R>> {
   Ok(csv::ReaderBuilder::new().delimiter(b'\t').from_reader(closes))
}

pub fn new_close_pivots<R: io::Read>(pivots: &[Pivot], closes: &mut Closes<R>)
      -> ErrStr<Vec<ClosePivot>> {
   let mut new_closes = Vec::new();

   let mut ix = 0;
   // 3. Process records and write new format
   for result in closes.deserialize() {
      ix += 1;
      let old_row: OldClosePivotRow = err_or(result,
           &format!("Cannot convert old close pivot row, ix: {ix}"))?;
      let pivots_for = filter_pivots(pivots, &old_row.open_pivots_ix());
      let gain_10 = size(&pivots_for) * 1.1;
      let (_, weighted_open_dt) =
         weighted_date(&pivots_for, &old_row.closed())?;
      let new_row = transform(&old_row, &weighted_open_dt, gain_10);
      new_closes.push(new_row);
   }
   Ok(new_closes)
}


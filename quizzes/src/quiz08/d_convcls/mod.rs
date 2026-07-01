use std::{ collections::HashMap, fs::File, io };
use csv::Reader;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use book::{
   currency::usd::USD,
   csv_utils::as_tsv,
   err_utils::{ErrStr,err_or},
   num::percentage::Percentage,
   string_utils::s,
   utils::get_args
};

// Maps only what we need from the open pivots table
#[derive(Debug, Deserialize)]
struct OpenPivotRow {
    #[serde(alias = "open")]
    pivot: String,
    close: String,
    #[serde(alias = "10% gain")]
    gain_10_percent: f32,
}

// Maps the incoming fields from the old close pivots table
#[serde_as]
#[derive(Debug, Deserialize)]
struct OldClosePivotRow {
    date: String,
    #[serde(alias = "open")]
    pivot: String,
    close: String,
    tx_id: String,
    from: String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(alias = "from quote")]
    from_quote: USD,
    to: String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(alias = "to quote")]
    to_quote: USD,
    trade: String,
    #[serde_as(as = "DisplayFromStr")]
    vol: USD,
    #[serde(alias = "new to-actual")]
    new_to_actual: String,
    gain: String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(alias = "gain, total $")]
    gain_total_usd: USD,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(alias = "ROI")]
    roi: Percentage,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(alias = "APR")]
    apr: Percentage,
}

// Defines your exact output layout

#[serde_as]
#[derive(Debug, Serialize)]
struct NewClosePivotRow {
    date: String,
    pivot: String,
    close: String,
    tx_id: String,
    from: String,
    #[serde_as(as = "DisplayFromStr")]
    from_quote: USD,
    to: String,
    #[serde_as(as = "DisplayFromStr")]
    to_quote: USD,
    trade: String,
    #[serde_as(as = "DisplayFromStr")]
    vol: USD,
    gain_10_percent: f32,
    new_to_actual: String,
    gain: String,
    #[serde_as(as = "DisplayFromStr")]
    gain_total_usd: USD,
    #[serde_as(as = "DisplayFromStr")]
    roi: Percentage,
    #[serde_as(as = "DisplayFromStr")]
    apr: Percentage,
}

fn version() -> String { s("1.00") }
fn app_name() -> String { s("convcls") }

fn usage() -> ErrStr<()> {
   let app = app_name();
   println!("{}, version: {}

Usage:

$ {} <open-pivots-filename> <close-pivots-filename>

Converts the old close-pivot format to the current close pivot format,
computing the 10% gains from the open pivot table.", app, version(), app);
   Err(s("convcls needs <opens> and <closes> pivot table file names."))
}

type Opens = HashMap<(String, String), f32>;
type Closes<R> = Reader<R>;

fn process_open_pivots<R: io::Read>(opens: R) -> ErrStr<Opens> {
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

fn process_old_close_pivots<R: io::Read>(closes: R) -> ErrStr<Closes<R>> {
    Ok(csv::ReaderBuilder::new().delimiter(b'\t').from_reader(closes))
}

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

fn new_close_pivots<R: io::Read>(opens: &Opens, closes: &mut Closes<R>)
      -> ErrStr<Vec<NewClosePivotRow>> {
   let mut new_closes = Vec::new();

   let mut ix = 0;
   // 3. Process records and write new format
   for result in closes.deserialize() {
      ix += 1;
      let old_row: OldClosePivotRow = err_or(result,
           &format!("Cannot convert old close pivot row, ix: {ix}"))?;
      let gain_10 = gain_10_percent_for_close(&opens, &old_row)?;
      let new_row = NewClosePivotRow {
            date: old_row.date,
            pivot: old_row.pivot,
            close: old_row.close,
            tx_id: old_row.tx_id,
            from: old_row.from,
            from_quote: old_row.from_quote,
            to: old_row.to,
            to_quote: old_row.to_quote,
            trade: old_row.trade,
            vol: old_row.vol,
            gain_10_percent: gain_10,
            new_to_actual: old_row.new_to_actual,
            gain: old_row.gain,
            gain_total_usd: old_row.gain_total_usd,
            roi: old_row.roi,
            apr: old_row.apr,
      };
      new_closes.push(new_row);
   }
   Ok(new_closes)
}

pub fn runoff_with_args() -> ErrStr<()> {
   if let [opens, closes] = get_args().as_slice() {
      let open_file = 
         err_or(File::open(opens),
                &format!("Cannot open open pivot table: {opens}"))?;
      let open_map = process_open_pivots(&open_file)?;
      let close_file = err_or(File::open(closes),
                 &format!("Cannot open old close pivot table: {closes}"))?;
      let mut close_rdr = process_old_close_pivots(&close_file)?;
      let closes = new_close_pivots(&open_map, &mut close_rdr)?;
      let table = as_tsv(&closes)?;
      println!("{table}");
      Ok(())
   } else { usage() }
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

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use super::sample_data::{sample_open_pivots,sample_old_close_pivots};
   use paste::paste;
   use book::create_testing;

   create_testing!("quizzes::quiz08::d_convcls");

   run!("process_open_pivots", {
      let open_tsv = sample_open_pivots();
      println!("Converting\n{open_tsv}");
      let open_pivots = process_open_pivots(open_tsv.as_bytes())?;
      println!("to hash\n{open_pivots:?}");
   });

   run!("new_close_pivot_table", {
      let open_tsv = sample_open_pivots();
      println!("Using\n{open_tsv}");
      let open_pivots = process_open_pivots(open_tsv.as_bytes())?;
      let old_closes = sample_old_close_pivots();
      println!("Converting\n{old_closes}");
      let mut basis = process_old_close_pivots(old_closes.as_bytes())?;
      let new_closes = new_close_pivots(&open_pivots, &mut basis)?;
      let table = as_tsv(&new_closes)?;
      println!("Result:\n{table}");
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use super::sample_data::{ sample_open_pivots, sample_old_close_pivots };

   fn to_str((a, b): (&str, &str)) -> (String, String) { (s(a), s(b)) }

   #[test] fn test_open_pivots() -> ErrStr<()> {
      let opens = process_open_pivots(sample_open_pivots().as_bytes())?;
      assert_eq!(4, opens.len(), "Need 2 parsed open pivots");
      assert_eq!(Some(&1939.31), opens.get(&to_str(("4", "2"))));
      Ok(())
   }

   #[test] fn test_old_close_pivots_ok() {
      let input = sample_old_close_pivots();
      let closes = process_old_close_pivots(input.as_bytes());
      assert!(closes.is_ok(), "Cannot parse old close pivots");
   }

   #[test] fn test_old_close_pivots_deserialize() -> ErrStr<()> {
      let input = sample_old_close_pivots();
      let mut closes = process_old_close_pivots(input.as_bytes())?;
      let mut x = 0;
      for close in closes.deserialize::<OldClosePivotRow>() {
         x += 1;
         assert!(close.is_ok(), "Old close pivot {x} parse failed");
      }
      assert_eq!(2, x, "Should have 2 old close pivots");
      Ok(())
   }

   #[test] fn test_new_close_pivots() -> ErrStr<()> {
      let opens = process_open_pivots(sample_open_pivots().as_bytes())?;
      let input = sample_old_close_pivots();
      let mut closes = process_old_close_pivots(input.as_bytes())?;
      let new_closes = new_close_pivots(&opens, &mut closes)?;
      assert_eq!(2, new_closes.len(), "There should be 2 new close pivots");
      assert_eq!(49.5, new_closes[0].gain_10_percent, "composite gain 10%");
      Ok(())
   }
}


use std::fs::File;
use clap::Parser;

use book::{
   parse_args_add_banner,
   cli_utils::add_banner,
   csv_utils::as_tsv,
   err_utils::{ErrStr,err_or}
};

use libs::{
   processors::pivots::{
      process_open_pivots,
      process_old_close_pivots,
      new_close_pivots
   }
};

/// Converts the old close-pivot format to the current close pivot format,
/// 
/// convcls computes the 10% gains from the open pivot table.
#[derive(Debug, Parser)]
#[command(name = "convcls")]
#[command(version = "1.01")]
struct Args {
   /// Path to the open pivots table
   opens: String,

   /// Path to the close pivots table
   closes: String
}

pub fn runoff_with_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   let opens = &args.opens;
   let open_file = 
      err_or(File::open(opens),
             &format!("Cannot open open pivot table: {opens}"))?;
   let open_map = process_open_pivots(&open_file)?;
   let closes = &args.closes;
   let close_file = err_or(File::open(closes),
              &format!("Cannot open old close pivot table: {closes}"))?;
   let mut close_rdr = process_old_close_pivots(&close_file)?;
   let closes = new_close_pivots(&open_map, &mut close_rdr)?;
   let table = as_tsv(&closes)?;
   println!("{table}");
   Ok(())
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
   use book::string_utils::s;
   use libs::types::pivots::closes::OldClosePivotRow;

   fn to_str((a, b): (&str, &str)) -> (String, String) { (s(a), s(b)) }

   #[test] fn test_open_pivots() -> ErrStr<()> {
      let opens = process_open_pivots(sample_open_pivots().as_bytes())?;
      assert_eq!(4, opens.len(), "Need 2 parsed open pivots");
      assert_eq!(Some(&1939.31), opens.get(&4));
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
      assert_eq!(49.5, new_closes[0].gain(), "composite gain 10%");
      Ok(())
   }
}


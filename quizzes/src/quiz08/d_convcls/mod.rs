use std::fs::File;
use clap::Parser;

use book::{
   parse_args_add_banner,
   cli_utils::generate_banner,
   csv_utils::as_tsv,
   err_utils::{ErrStr,err_or}
};

use libs::{
   fetchers::pivots::read_pivots,
   processors::pivots::closes::{
      process_old_close_pivots,
      new_close_pivots
   },
   types::{ aliases::aliases, pivots::opens::Pivot }
};

/// Converts the old close-pivot format to the current close pivot format,
/// 
/// convcls computes the 10% gains from the open pivot table.
#[derive(Debug, Parser)]
#[command(name = "convcls")]
#[command(version = "1.04")]
struct Args {
   /// Path to the open pivots table
   opens: String,

   /// Path to the close pivots table
   closes: String,

   /// Print debugging information
   #[arg(short, long)]
   debug: bool
}

pub fn runoff_with_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   let a = aliases();
   let (opens, _mx_dt) = read_pivots(&args.opens, &a, args.debug)?;
   let closes = &args.closes;
   with_open_pivots(&opens, closes)
}

fn with_open_pivots(opens: &[Pivot], closes: &str) -> ErrStr<()> {
   let close_file = err_or(File::open(closes),
              &format!("Cannot open old close pivot table: {closes}"))?;
   let mut close_rdr = process_old_close_pivots(&close_file)?;
   let closes = new_close_pivots(&opens, &mut close_rdr)?;
   let table = as_tsv(&closes, true)?;
   println!("{table}");
   Ok(())
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, file_utils::read_file };
   use libs::fetchers::pivots::sample_reader::read_sample_open_pivots;

   create_testing!("quizzes::quiz08::d_convcls");

   run!("new_close_pivot_table", {
      let open_pivots =
         read_sample_open_pivots("data/sample_avax_undead_open_pivots.tsv",
                                 "avax-undead")?;
      println!("Using AVAX+UNDEAD open pivots");
      let old_closes =
        read_file("data/sample_old_close_avax_undead_pivot.tsv")?;
      println!("Converting AVAX+UNDEAD (old) close pivots");
      let mut basis = process_old_close_pivots(old_closes.as_bytes())?;
      let new_closes = new_close_pivots(&open_pivots, &mut basis)?;
      let table = as_tsv(&new_closes, true)?;
      println!("Result:\n{table}");
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use libs::{
      fetchers::pivots::sample_reader::read_sample_open_pivots,
      types::{ gains::Gains, pivots::closes::OldClosePivotRow }
   };
   use book::{ file_utils::read_file, num::estimate::mk_estimate };

   #[test] fn test_old_close_pivots_ok() -> ErrStr<()> {
      let old_closes =
        read_file("data/sample_old_close_avax_undead_pivot.tsv")?;
      let closes = process_old_close_pivots(old_closes.as_bytes());
      assert!(closes.is_ok(), "Cannot parse old close pivots");
      Ok(())
   }

   #[test] fn test_old_close_pivots_deserialize() -> ErrStr<()> {
      let old_closes =
         read_file("data/sample_old_close_avax_undead_pivot.tsv")?;
      let mut closes = process_old_close_pivots(old_closes.as_bytes())?;
      let mut x = 0;
      for close in closes.deserialize::<OldClosePivotRow>() {
         x += 1;
         assert!(close.is_ok(), "Old close pivot {x} parse failed");
      }
      assert_eq!(4, x, "Should have 2 old close pivots");
      Ok(())
   }

   #[test] fn test_new_close_pivots() -> ErrStr<()> {
      let opens =
         read_sample_open_pivots("data/sample_avax_undead_open_pivots.tsv",
                                 "avax-undead")?;
      let old_closes =
         read_file("data/sample_old_close_avax_undead_pivot.tsv")?;
      let mut closes = process_old_close_pivots(old_closes.as_bytes())?;
      let new_closes = new_close_pivots(&opens, &mut closes)?;
      assert_eq!(4, new_closes.len(), "There should be 2 new close pivots");
      let gain_est = mk_estimate(3.85);
      gain_est.is(new_closes[0].gain())
   }
}

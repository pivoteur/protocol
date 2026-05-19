use std::fs::OpenOptions;
use std::io::{self, BufRead, Write};

use book::{
   err_utils::ErrStr,
   list_utils::{ first_last, tail },
   stream_utils::lines_from_stdio
};

use libs::tables::c2t;

pub fn write_file_from_stdio() -> ErrStr<()> {
   let lines = lines_from_stdio();
   write_file_from(&lines)
}

fn write_file_from(lines: &[&str]) -> ErrStr<()> {
   let relevant = tail(lines);
   if let (Some(line), Some(filename)) = first_last(&relevant) {
      append_close(filename, line)
   } else {
      Err(format!("stream must contain close pivot and filename; stream:
{}", lines.join("\n")))
   }
}

fn append_close(filename: &str, close_pivot: &str) -> ErrStr<()> {
   let trimmed_filename = filename.trim();
   let mut file =
      OpenOptions::new()
            .create(false)
            .write(true)
            .append(true)
            .open(trimmed_filename)?;
   err_or(writeln!(file, "{}", c2t(close_pivot)),
          format!("Cannot write to file {trimmed_filename}"))
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;

   use paste::paste;
   use book::create_testing;

   create_testing!("quiz08::c_appender");

   run!("write_file_from", xxx

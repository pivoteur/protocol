use std::{ fmt::Debug, fs::OpenOptions, io::Write };

use book::{
   err_utils::{ err_or, ErrStr },
   list_utils::tail,
   stream_utils::lines_from_stdin
};

use libs::tables::c2t;

pub fn write_file_from_stdin() -> ErrStr<()> {
   let lines = lines_from_stdin()?;
   write_file_from(&lines)
}

fn fst_snd<T: Clone + Debug>(list: &[T]) -> ErrStr<(T, T)> {
   fn firstly<U: Clone + Debug>(lst: &[U]) -> ErrStr<(U, Vec<U>)> {
      let a = lst.first().ok_or(format!("Cannot first() this list {lst:?}"))?;
      Ok((a.clone(), tail(lst)))
   }
   let (a, rest) = firstly(list)?;
   let (b, _) = firstly(&rest)?;
   Ok((a, b))
}

fn write_file_from(lines: &[String]) -> ErrStr<()> {
   let relevant = tail(lines);
   let (line, filename) = fst_snd(&relevant)?;
   append_close(&filename, &line)
}

fn append_close(filename: &str, close_pivot: &str) -> ErrStr<()> {
   let tfile = filename.trim();
   let mut file =
      err_or(OpenOptions::new()
            .create(false)
            .write(true)
            .append(true)
            .open(tfile), &format!("Cannot open file {tfile}"))?;
   err_or(writeln!(file, "{}", c2t(close_pivot)),
          &format!("Cannot write to file {tfile}"))
}

pub fn runoff_with_args() -> ErrStr<()> { write_file_from_stdin() }

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod sample_close_pivot {
   use book::string_utils::lines;

   pub const FILE: &str = "data/sample_close_pivots.tsv";
   pub fn close_pivot(file: &str) -> Vec<String> {
      lines(&format!("date,pivot,close,tx_id,from,from_quote,to,to_quote,trade,vol,gain_10_percent,new_to_actual,gain,gain_total_usd,roi,apr
2026-05-18,46,14,asdf,BTC,$76772,ETH,$2114.42,0.00467,$358.5252,0.1648,0.169101199810017841,0.0193,$40.81,12.88%,24.24%
{file}
"))
   }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use super::sample_close_pivot::{ FILE, close_pivot };
   use paste::paste;
   use book::{ create_testing, file_utils::lines_from_file };

   create_testing!("quiz08::c_appender");

   run!("write_file_from", {
      let piv = &close_pivot(FILE);
      let my_closes = lines_from_file(FILE)?;
      println!("My original close pivots:\n\n{}", my_closes.join("\n"));
      write_file_from(piv)?;
      let more_closes = lines_from_file(FILE)?;
      println!("My new close pivots:\n\n{}", more_closes.join("\n"));
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use super::sample_close_pivot::{ FILE, close_pivot };
   use book::file_utils::lines_from_file;

   #[test] fn fail_write_file_from() {
      let piv = &close_pivot("data/ur_mom.txt");
      let ans = write_file_from(piv);
      assert!(ans.is_err());
   }

   #[test] fn test_write_file_from() {
      let piv = &close_pivot(FILE);
      let ans = write_file_from(piv);
      assert!(ans.is_ok());
   }

   #[test] fn test_write_file_from_adds_row() -> ErrStr<()> {
      let my_closes = lines_from_file(FILE)?;
      let piv = &close_pivot(FILE);
      write_file_from(piv)?;
      let new_closes = lines_from_file(FILE)?;
      assert_eq!(my_closes.len() + 1, new_closes.len());
      Ok(())
   }
}


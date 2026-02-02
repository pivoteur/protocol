use book::{
   err_utils::ErrStr,
   list_utils::ht,
   parse_utils::{parse_id,parse_str},
   table_utils::{Table,ingest}
};

use super::types::util::Id;

/// From a set of row-data, index the rows and parse into a table
pub fn index_table(lines: Vec<String>) -> ErrStr<Table<Id, String, String>> {
   let (h, t) = ht(&lines);
   let h1 = h.ok_or("empty list for data set")?;
   let header = format!("ix\t{h1}");
   let mut body: Vec<String> =
      t.iter().enumerate().map(|(a, b)| format!("{a}\t{b}")).collect();
   body.insert(0, header);
   ingest(parse_id, parse_str, parse_str, &body, "\t")
}

pub mod functional_tests {

   use super::*;

   use book::{csv_utils::CsvWriter,string_utils::to_string};

   pub fn some_rows() -> Vec<String> {
      vec![
"date	count	priority	status	color",
"2026-01-02	1	a	3	blue",
"2026-01-02	3	b	7	green",
"2026-03-14	7	c	1	red",
"2026-05-07	11	d	2	indigo"
          ].into_iter().map(to_string).collect()
   }

   pub fn runoff() -> ErrStr<usize> {
      println!("\ntables functional test\n");
      let table = index_table(some_rows())?;
      println!("Indexed table is:\n\n{}", table.as_csv());
      Ok(1)
   }
}

#[cfg(test)]
mod tests {

   use super::*;

   use book::table_utils::rows;

   #[test]
   fn test_index_table_ok() {
      let mb_table = index_table(functional_tests::some_rows());
      assert!(mb_table.is_ok());
   }

   #[test]
   fn test_index_table_rows_same_number_as_input_data() -> ErrStr<()> {
      let table = index_table(functional_tests::some_rows())?;
      assert_eq!(functional_tests::some_rows().len() - 1, rows(&table).len());
      Ok(())
   }
}


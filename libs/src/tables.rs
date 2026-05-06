use std::iter::once;

use book::{
   csv_utils::CsvWriter,
   err_utils::ErrStr,
   list_utils::ht,
   parse_utils::{parse_id,parse_str},
   string_utils::to_string,
   table_utils::{Table,ingest,cols}
};

use super::types::util::Id;

pub type IxTable = Table<Id, String, String>;

/// From a set of row-data, index the rows and parse into a table
pub fn index_table(lines: Vec<String>) -> ErrStr<IxTable> {
   index_table_sep("\t", lines)
}

pub fn index_table_sep(sep: &str, lines: Vec<String>) -> ErrStr<IxTable> {
   let (h, t) = ht(&lines);
   let h1 = h.ok_or("empty list for data set")?;
   let header = format!("ix{sep}{h1}");
   let mut body: Vec<String> =
      t.iter().enumerate().map(|(a, b)| format!("{a}\t{b}")).collect();
   body.insert(0, header);
   ingest(parse_id, parse_str, parse_str, &body, sep)
}

pub fn sans_index(t: &IxTable) -> Vec<String> {
   let hdr = tabify(&cols(t));
   let ans: Vec<String> = once(hdr).chain(t.data.iter().map(tabify)).collect();
   ans
}

pub fn csv2tsv<T:CsvWriter>(row: &T) -> String {
   c2t(&row.as_csv())
}

fn underscore(s: String) -> String {
   let ss: Vec<String> = s.split(" ").map(to_string).collect();
   ss.join("_")
}

pub fn c2t(row: &str) -> String {
   let cols: Vec<String> = row.split(",").map(to_string).collect();
   let under_cols: Vec<String> = cols.into_iter().map(underscore).collect();
   tabify(&under_cols)
}

fn tabify(row: &Vec<String>) -> String {
   row.join("\t")
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {

   use super::*;
   use paste::paste;

   use book::{
      create_testing,
      csv_utils::CsvWriter,
      string_utils::s,
      utils::{composer,deref}
   };

   create_testing!("tables");

   pub fn some_rows() -> Vec<String> {
      vec![
"date	count	priority	status	color",
"2026-01-02	1	a	3	blue",
"2026-01-02	3	b	7	green",
"2026-03-14	7	c	1	peach blossom",
"2026-05-07	11	d	2	indigo"
          ].into_iter().map(to_string).collect()
   }

   fn rows(v: Vec<String>) -> String { v.join("\n") }
   fn table() -> IxTable { index_table(some_rows()).unwrap() }
   run_with!("underscore", s("peanut butter"), underscore);
   run_with!("indexed_table", &table(), CsvWriter::as_csv);
   run_with!("sans_index", table(), composer(rows, deref(sans_index)));
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

   #[test]
   fn test_sans_index() -> ErrStr<()> {
      let table = index_table(functional_tests::some_rows())?;
      let tsv = sans_index(&table);
      assert_eq!(functional_tests::some_rows().len(), tsv.len());
      Ok(())
   }

   #[test]
   fn test_reingest_from_sans_index_ok() -> ErrStr<()> {
      let table = index_table(functional_tests::some_rows())?;
      let tsv = sans_index(&table);
      let table1 = index_table(tsv);
      assert!(table1.is_ok());
      Ok(())
   }

   #[test]
   fn test_reingest_from_sans_index_same_size() -> ErrStr<()> {
      let table = index_table(functional_tests::some_rows())?;
      let tsv = sans_index(&table);
      let table1 = index_table(tsv)?;
      assert_eq!(table.data.len(), table1.data.len());
      Ok(())
   }

   #[test]
   fn test_reingest_from_sans_index_same_headers() -> ErrStr<()> {
      let table = index_table(functional_tests::some_rows())?;
      let tsv = sans_index(&table);
      let table1 = index_table(tsv)?;
      assert_eq!(cols(&table), cols(&table1));
      Ok(())
   }

   #[test]
   fn test_reingest_from_sans_index_same_row() -> ErrStr<()> {
      let table = index_table(functional_tests::some_rows())?;
      let tsv = sans_index(&table);
      let table1 = index_table(tsv)?;
      assert_eq!(table.data[2], table1.data[2]);
      Ok(())
   }

   #[test]
   fn test_underscore_idempotent() {
      let r = "red".to_string();
      let b = underscore(r.clone());
      assert_eq!(b,r);
   }

   #[test]
   fn test_underscore_replace() {
      let pb = "peach blossom".to_string();
      let bp = underscore(pb);
      assert!(bp.contains("_"));
   }
}


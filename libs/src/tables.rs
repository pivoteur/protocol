use book::{
   err_utils::ErrStr,
   list_utils::ht,
   parse_utils::{parse_id,parse_str},
   table_utils::{Table,ingest}
};

use crate::types::util::Id;

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


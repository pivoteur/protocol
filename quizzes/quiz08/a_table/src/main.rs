use book::{
   csv_utils::CsvWriter,
   err_utils::ErrStr,
   stream_utils::lines_from_stream,
   table_utils::{ingest,val},
   utils::get_args
};

use libs::parsers::{parse_id,parse_str};

fn main() -> ErrStr<()> {
   if let [row, col] = get_args().as_slice() {
      let r = parse_id(&row)?;
      let lines = lines_from_stream();
      let body: Vec<String> = lines.into_iter().skip(4).collect();

      let table = ingest(parse_id, parse_str, parse_str, &body, ",")?;
      println!("Table is:\n\n{}", table.as_csv());
      println!("\nThe value at row {} / col {} is {:?}",
               r, col, val(&table, &r, &col));
      Ok(())
   } else {
      Err("Select a <row> and <col> view".to_string())
   }
}

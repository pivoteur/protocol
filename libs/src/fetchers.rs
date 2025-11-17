/// fetch data from REST endpoints

use book::{
   err_utils::ErrStr,
   rest_utils::read_rest,
   table_utils::cols,
   utils::pred
};
use crate::{
   parsers::enum_headers,
   paths::open_pivot_path,
   tables::index_table,
   types::pivots::{Pivot,parse_pivot,active}
};

pub async fn fetch_pivots(primary: &str, pivot: &str)
      -> ErrStr<(Vec<Pivot>, Vec<Pivot>)> {
   let pri = primary.to_lowercase();
   let seggs = pivot.to_lowercase();
   let url = open_pivot_path(&pri, &seggs);
   let lines = fetch_lines(&url).await?;
   let table = index_table(lines)?;

   let hdrs = enum_headers(cols(&table));

   let mut acts: Vec<Pivot> = Vec::new();
   let mut pass: Vec<Pivot> = Vec::new();

   for row in table.data {
      let piv = parse_pivot(&hdrs, &row)?;
      if active(&piv) {
         acts.push(piv.clone());
      } else {
         pass.push(piv);
      }
   }
   Ok((acts, pass))
}

pub async fn fetch_open_pivots(primary: &str, pivot: &str)
      -> ErrStr<Vec<Pivot>> {
   let (ans, _) = fetch_pivots(primary, pivot).await?;
   Ok(ans)
}

async fn fetch_lines(url: &str) -> ErrStr<Vec<String>> {
   let daters = read_rest(url).await?;
   let lines: Vec<String> =
      daters.lines()
            .filter_map(|l| pred(!l.is_empty(), l.to_string()))
            .collect();
   Ok(lines)
}

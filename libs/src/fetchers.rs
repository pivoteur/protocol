/// fetch data from REST endpoints

use std::collections::HashMap;

use chrono::NaiveDate;

use book::{
   currency::usd::USD,
   date_utils::{parse_date,datef},
   err_utils::{err_or,ErrStr},
   list_utils::tail,
   num_utils::{parse_num,parse_commaless},
   rest_utils::read_rest,
   table_utils::{Table,cols,row,rows,ingest},
   utils::pred
};
use super::{
   parsers::{parse_str,enum_headers},
   paths::{open_pivot_path,quotes_url,pool_assets_url},
   tables::index_table,
   types::{
      aliases::aliases,
      pivots::{Pivot,parse_pivot,active},
      quotes::{Quotes,mk_quotes},
      assets::{Asset,mk_asset},
      comps::{Composition,mk_composition},
      util::{Token,Blockchain}
   }
};

pub async fn fetch_assets(root_url: &str, primary: &str, pivot: &str)
      -> ErrStr<Composition> {
   let (pri, seggs) = enlowerify(primary, pivot);
   let url = pool_assets_url(root_url, &pri, &seggs);
   let lines = fetch_lines(&url).await?;
   let table = ingest(parse_date, parse_str, parse_str, &lines, "\t")?;
   let hdrs = enum_headers(cols(&table));
   let (p, s) = enupperify(primary, pivot);
   let max_date = rows(&table).iter().max().cloned()
                              .ok_or(format!("No max_date for {p}+{s}"))?;
   let top = row(&table, &max_date)
                .ok_or(format!("No row for date {max_date}"))?;
   let blk = top[hdrs["blockchain"]].clone();
   let primary = buidl_asset(&top[hdrs[&p]], qt_f(&top, &hdrs),
                             &blk, &p, &max_date)?;
   let pivot = buidl_asset(&top[hdrs[&s]], qt_f(&top, &hdrs),
                             &blk, &s, &max_date)?;
   Ok(mk_composition(primary, pivot))
}

fn qt_f<'a>(v: &'a Vec<String>, hdrs: &'a HashMap<String, usize>)
      -> impl Fn(&'a Token) -> ErrStr<USD> {
   |t: &'a Token| {
      let q = &v[hdrs[&format!("{t} quote")]];
      let quote: USD = err_or(q.parse(), &format!("No quote for {t}"))?;
      Ok(quote)
   }
}
   
fn buidl_asset<'a>(amount: &str, q: impl Fn(&'a Token) -> ErrStr<USD>, 
                   blk: &Blockchain, t: &'a Token, dt: &NaiveDate)
      -> ErrStr<Asset> {
   let amt = parse_commaless(amount)?;
   let quote = q(t)?;
   Ok(mk_asset(&(blk.clone(), t.clone()), amt, &quote, dt))
}

/// Fetch the pivots for pivot pool A+B; open pivots are reposed in git
pub async fn fetch_pivots(root_url: &str, primary: &str, pivot: &str)
      -> ErrStr<(Vec<Pivot>, Vec<Pivot>, NaiveDate)> {
   let (pri, seggs) = enlowerify(primary, pivot);
   let pool = format!("{pri}+{seggs}");
   let url = open_pivot_path(root_url, &pri, &seggs);
   let lines = fetch_lines(&url).await?;
   let table = index_table(lines)?;

   let hdrs = enum_headers(cols(&table));

   let max_date = max_diem(&table, hdrs["opened"], &pool)?;
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
   Ok((acts, pass, max_date.clone()))
}

fn enlowerify(primary: &str, pivot: &str) -> (String, String) {
   (primary.to_lowercase(), pivot.to_lowercase())
}

fn enupperify(primary: &str, pivot: &str) -> (String, String) {
   let a = aliases();
   (a.alias(primary), a.alias(pivot))
}

fn max_diem<T>(table: &Table<T, String, String>, ix: usize, pool: &str)
      -> ErrStr<NaiveDate> {
   table.data
        .iter()
        .map(|row| datef(&row[ix]))
        .max()
        .ok_or(format!("No max date for {pool} pivot pool"))
}

/// Filter to only the open pivots for pivot pool A+B
pub async fn fetch_open_pivots(root_url: &str, primary: &str, pivot: &str)
      -> ErrStr<(Vec<Pivot>, NaiveDate)> {
   let (ans, _, max_date) = fetch_pivots(root_url, primary, pivot).await?;
   Ok((ans, max_date))
}

async fn fetch_lines(url: &str) -> ErrStr<Vec<String>> {
   let daters = read_rest(url).await?;
   let lines: Vec<String> =
      daters.lines()
            .filter_map(|l| pred(!l.is_empty(), l.to_string()))
            .collect();
   Ok(lines)
}

/// fetch the quotes for date; historical quote-data is reposed in git
pub async fn fetch_quotes(date: &NaiveDate) -> ErrStr<Quotes> {
   let lines = fetch_lines(&quotes_url()).await?;
   let body: Vec<String> = tail(&lines);
   let table = ingest(parse_date, parse_str, parse_str, &body, ",")?;
   if let Some(quotes_row) = row(&table, date) {
      let mut quotes = HashMap::new();
      let hdrs = cols(&table);
      for (n, h) in hdrs.iter().enumerate() {
         let qt: f32 = parse_num(&quotes_row[n])?;
         quotes.insert(h.clone(), qt);
      }
      Ok(mk_quotes(date.clone(), quotes))
   } else {
      Err(format!("Unable to find quotes for date {date}"))
   }
}


use std::{collections::HashMap, hash::Hash};

use chrono::NaiveDate;

use book::{
   date_utils::parse_date,
   err_utils::{ErrStr,err_or},
   list_utils::ht,
   rest_utils::read_rest,
   table_utils::{ingest,cols},
   tuple_utils::swap,
   utils::pred
};

use pivots::paths::open_pivot_path;

fn parse_int(s: &str) -> ErrStr<i32> {
   err_or(s.parse(), &format!("{s} is not an integer"))
}

fn parse_str(s: &str) -> ErrStr<String> {
   Ok(s.to_string())
}

type Id = usize;

#[derive(Debug, Clone)]
struct Header {
   opened: NaiveDate,
   id: Id,
   close: Id
}

fn mk_hdr(opend: &str, id: Id, close: Id) -> ErrStr<Header> {
   let opened = parse_date(opend)?;
   Ok(Header { opened, id, close })
}

#[derive(Debug, Clone)]
struct Amount {
   actual: f32,
   ersatz: f32      // 'ersatz' meaning 'virtual' as 'virtual' is reserved
}

fn amount(a: Amount) -> f32 { a.actual + a.ersatz }
fn mk_amt(actual: f32, ersatz: f32) -> Amount {
   Amount { actual, ersatz }
}

#[derive(Debug, Clone)]
struct Asset {
   token: String,
   amount: Amount
}

fn mk_asset(tkn: &str, amount: Amount) -> Asset {
   Asset { token: tkn.to_string(), amount }
}

/// Defines the structure of an open pivot
#[derive(Debug, Clone)]
struct Pivot {
   header: Header,
   from: Asset,
   to: Asset
}

fn closed(p: &Pivot) -> bool {
   p.header.close > 0
}
fn active(p: &Pivot) -> bool {
   !closed(p)
}

fn sample_pivot_0(dt: &str) -> ErrStr<Pivot> {
   let header = mk_hdr(dt, 1, 0)?;
   let from = mk_asset("BTC", mk_amt(0.004498, 0.0));
   let to = mk_asset("ETH", mk_amt(0.14203, 0.0));
   Ok(Pivot { header, from, to })
}

fn sample_pivot() -> ErrStr<Pivot> {
   sample_pivot_0("2025-11-10")
}

fn mk_pivot_0(hdrs: &HashMap<String, usize>, row: &Vec<String>)
      -> ErrStr<Pivot> {
   sample_pivot_0(&row[hdrs["opened"]])
}

fn enum_headers<HEADER: Eq + Hash>(headers: Vec<HEADER>)
      -> HashMap<HEADER, usize> {
   headers.into_iter().enumerate().map(swap).collect()
}

#[tokio::main]
async fn main() -> ErrStr<()> {
   // println!("Voilà: {:?}", sample_pivot());

// now, let's read in real open pivot data and first, put those data into a 
// (unstructured) table

   let url = open_pivot_path("btc", "eth");
   let daters = read_rest(&url).await?;
   let lines: Vec<String> =
      daters.lines()
            .filter_map(|l| pred(!l.is_empty(), l.to_string()))
            .collect();
   let (h, t) = ht(&lines);
   let h1 = h.ok_or("empty list for data set")?;
   let header = format!("ix\t{h1}");
   let mut body: Vec<String> =
      t.iter().enumerate().map(|(a, b)| format!("{a}\t{b}")).collect();
   body.insert(0, header);
   let table = ingest(parse_int, parse_str, parse_str, &body, "\t")?;
   let hdrs = enum_headers(cols(&table));
   for row in table.data {
      let piv = mk_pivot_0(&hdrs, &row)?;
      println!("row: {piv:?}");
   }
   Ok(())
}

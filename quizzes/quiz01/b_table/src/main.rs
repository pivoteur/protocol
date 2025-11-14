use std::{collections::HashMap, hash::Hash};

use chrono::NaiveDate;

use book::{
   currency::usd::{USD,mk_usd},
   date_utils::parse_date,
   err_utils::{ErrStr,err_or},
   list_utils::ht,
   num_utils::parse_num,
   rest_utils::read_rest,
   table_utils::{ingest,cols},
   tuple_utils::swap,
   utils::pred
};

use pivots::paths::open_pivot_path;

fn parse_int(s: &str) -> ErrStr<usize> {
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
   amount: Amount,
   quote: USD
}

fn mk_asset(tkn: &str, amount: Amount, quote: USD) -> Asset {
   Asset { token: tkn.to_string(), amount, quote }
}

fn slice2vec(ss: &[&str]) -> Vec<String> {
   let mut vec: Vec<String> = Vec::new();
   for s in ss {
      vec.push(s.to_string());
   }
   vec
}

enum AssetType { FROM, TO }

impl AssetType {
   fn keys(&self) -> Vec<String> {
      match self {
         AssetType::FROM => slice2vec(&["from","amount1","virtual","quote1"]),
         AssetType::TO => slice2vec(&["to","net","blah!","quote2"])
      }
   }
}

fn parse_asset(a: AssetType, hdrs: &HashMap<String, usize>, row: &Vec<String>)
      -> ErrStr<Asset> {
   let keys = a.keys();
   if let [tok, amt, virt, qut] = keys.as_slice() {
      let token = &row[hdrs[tok]];
      let amnt = parse_num(&row[hdrs[amt]])?;
      let vrt = if let Some(virt_key) = hdrs.get(virt) {
         parse_num(&row[*virt_key])
      } else { Ok( 0.0 ) }?;
      let quot: USD = row[hdrs[qut]].parse()?;
      let amount = mk_amt(amnt, vrt);
      Ok(mk_asset(token, amount, quot))
   } else {
      Err("bad pattern match in AssetType enum for keys()".to_string())
   }
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

fn sample_pivot_1(header: Header) -> ErrStr<Pivot> {
   let from = mk_asset("BTC", mk_amt(0.004498, 0.0), mk_usd(99616.88));
   let to = mk_asset("ETH", mk_amt(0.14203, 0.0), mk_usd(3237.42));
   Ok(Pivot { header, from, to })
}

fn sample_pivot_0(dt: &str) -> ErrStr<Pivot> {
   let header = mk_hdr(dt, 1, 0)?;
   sample_pivot_1(header)
}

fn sample_pivot() -> ErrStr<Pivot> {
   sample_pivot_0("2025-11-10")
}

fn mk_pivot_0(hdrs: &HashMap<String, usize>, row: &Vec<String>)
      -> ErrStr<Pivot> {
   sample_pivot_0(&row[hdrs["opened"]])
}

fn mk_lookup_f(hdrs: &HashMap<String, usize>, row: &Vec<String>)
      -> impl Fn(String) -> String {
   move |key: String| {
      let col = hdrs[&key];
      row[col].clone()
   }
}

fn parse_header(hdrs: &HashMap<String, usize>, row: &Vec<String>)
      -> ErrStr<Header> {
   let lookf = mk_lookup_f(hdrs, row);
   fn looker<'a>(f: impl Fn(String) -> String + 'a)
         -> impl Fn(&'a str) -> String + 'a {
      move |key| f(key.to_string())
   }
   let look = looker(lookf);
   let dt = look("opened");
   let id = parse_int(&look("open"))?;
   let closed = parse_int(&look("close"))?;
   mk_hdr(&dt, id, closed)
}

fn mk_pivot_1(hdrs: &HashMap<String, usize>, row: &Vec<String>)
      -> ErrStr<Pivot> {
   let header = parse_header(hdrs, row)?;
   sample_pivot_1(header)
}

fn mk_pivot_2(hdrs: &HashMap<String, usize>, row: &Vec<String>)
      -> ErrStr<Pivot> {
   let header = parse_header(hdrs, row)?;
   let from = parse_asset(AssetType::FROM, hdrs, row)?;
   let to = parse_asset(AssetType::TO, hdrs, row)?;
   Ok( Pivot { header, from, to } )
}
   
fn enum_headers<HEADER: Eq + Hash>(headers: Vec<HEADER>)
      -> HashMap<HEADER, usize> {
   headers.into_iter().enumerate().map(swap).collect()
}

#[tokio::main]
async fn main() -> ErrStr<()> {
   // println!("Voilà: {:?}", sample_pivot());

// let's read in real open pivot data and first, put those data into a 
// (untyped) table

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

// We have our (unstructured) pivots tablized, now let's reify those pivots
// (... starting with just the opened-date data ... and now adding the rest of
// the header).

// Let's also parse the FROM- and TO-assets.

   let hdrs = enum_headers(cols(&table));

   let mut acts: Vec<Pivot> = Vec::new();
   let mut pass: Vec<Pivot> = Vec::new();

   let mut x: i32 = 1;
   for row in table.data {
      let piv = mk_pivot_2(&hdrs, &row)?;
      if active(&piv) {
         acts.push(piv.clone());
         println!("row {x}: {piv:?}");
         x = x + 1;
      } else {
         pass.push(piv);
      }
   }

   let a = acts.len();
   let p = pass.len();
   println!("\nThere are {a} active pivots and {p} closed pivots.");

   Ok(())
}

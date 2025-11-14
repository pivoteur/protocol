use std::{collections::HashMap, hash::Hash};

use chrono::NaiveDate;

use book::{
   currency::usd::{USD,mk_usd},
   csv_utils::CsvWriter,
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

trait CsvHeader {
   fn header(&self) -> String;
}

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

impl CsvWriter for Header {
   fn ncols(&self) -> usize { 3 }
   fn as_csv(&self) -> String {
      format!("{},{},{}", self.opened,self.id,self.close)
   }
}
impl CsvHeader for Header {
   fn header(&self) -> String { "opened,id,close_id".to_string() }
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

impl CsvWriter for Amount {
   fn ncols(&self) -> usize { 2 }
   fn as_csv(&self) -> String { format!("{},{}", self.actual, self.ersatz) }
}
impl CsvHeader for Amount {
   fn header(&self) -> String { "actual,virtual".to_string() }
}

fn amount(a: &Amount) -> f32 { a.actual + a.ersatz }
fn mk_amt(actual: f32, ersatz: f32) -> Amount {
   Amount { actual, ersatz }
}

#[derive(Debug, Clone)]
struct Asset {
   token: String,
   amount: Amount,
   quote: USD
}

impl CsvWriter for Asset {
   fn ncols(&self) -> usize { 1 + self.amount.ncols() + 1 + 1}
   fn as_csv(&self) -> String {
      let total = mk_usd(self.quote.amount * amount(&self.amount));
      format!("{},{},{},{}", self.token,self.amount.as_csv(),self.quote,total)
   }
}
impl CsvHeader for Asset {
   fn header(&self) -> String {
      format!("token,{},quote,total", self.amount.header()) 
   }
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

impl CsvWriter for Pivot {
   fn ncols(&self) -> usize { 
      self.header.ncols() + self.from.ncols() + self.to.ncols() + 1
   }
   fn as_csv(&self) -> String {
      let gain: f32 = amount(&self.from.amount) * 1.1;
      format!("{},{},{},{}", 
              self.header.as_csv(),
              self.from.as_csv(), gain,
              self.to.as_csv())
   }
}
impl CsvHeader for Pivot {
   fn header(&self) -> String {
      format!("{},{},gain_10_percent,{}",
              self.header.header(), self.from.header(), self.to.header())
   }
}

fn closed(p: &Pivot) -> bool {
   p.header.close > 0
}
fn active(p: &Pivot) -> bool {
   !closed(p)
}

/*
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
*/

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

/*
fn mk_pivot_1(hdrs: &HashMap<String, usize>, row: &Vec<String>)
      -> ErrStr<Pivot> {
   let header = parse_header(hdrs, row)?;
   sample_pivot_1(header)
}
*/

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

struct Bag<T> {
    counts: HashMap<T, usize>,
}

impl<T: Eq + std::hash::Hash> Bag<T> {
    fn new() -> Self {
        Bag {
            counts: HashMap::new(),
        }
    }

    fn add(&mut self, item: T) {
        *self.counts.entry(item).or_insert(0) += 1;
    }

/*
    fn count(&self, item: &T) -> usize {
        *self.counts.get(item).unwrap_or(&0)
    }

    fn remove(&mut self, item: &T) {
        if let Some(count) = self.counts.get_mut(item) {
            if *count > 1 {
                *count -= 1;
            } else {
                self.counts.remove(item);
            }
        }
    }
 */
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

   let mut btc: f32 = 0.0;
   let mut eth: f32 = 0.0;
   let mut bag: Bag<String> = Bag::new();
   let mut print_header: bool = true;
   let mut x: i32 = 1;
   for row in table.data {
      let piv = mk_pivot_2(&hdrs, &row)?;
      if active(&piv) {
         acts.push(piv.clone());
         if print_header {
            println!("ix,{}", piv.header());
            print_header = false;
         }
         let tok = piv.from.token.clone();
         let amt: f32 = amount(&piv.from.amount);
         if &tok == "BTC" {
            btc += amt;
         } else {
            eth += amt;
         }
         bag.add(tok);
         println!("{x},{}", piv.as_csv());
         x = x + 1;
      } else {
         pass.push(piv);
      }
   }

   let a = acts.len();
   let p = pass.len();
   println!("\nThere are {a} active pivots and {p} closed pivots.\n");

   for (k,v) in bag.counts {
      let amt: f32 = if k == "BTC" { btc } else { eth };
      println!("There are {v} {k} open pivots, totaling {amt} {k}");
   }

   Ok(())
}

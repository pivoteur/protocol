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
   table_utils::{Table,ingest,cols},
   tuple_utils::{swap,Partition},
   utils::{get_env,pred}
};

use libs::paths::open_pivot_path;

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
pub struct Header {
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

pub fn mk_hdr(opend: &str, id: Id, close: Id) -> ErrStr<Header> {
   let opened = parse_date(opend)?;
   Ok(Header { opened, id, close })
}

#[derive(Debug, Clone)]
pub struct Amount {
   actual: f32,
   ersatz: f32      // 'ersatz' meaning 'virtual' as 'virtual' is reserved
}

impl CsvWriter for Amount {
   fn ncols(&self) -> usize { 2 }
   fn as_csv(&self) -> String { format!("{},{}", self.actual, self.ersatz) }
}

fn kinderize(k: &AssetType, s: &[&str]) -> Vec<String> {
   s.iter().map(|elt| format!("{}_{}", k.kind(), elt)).collect()
}

pub fn amount(a: &Amount) -> f32 { a.actual + a.ersatz }
fn mk_amt(actual: f32, ersatz: f32) -> Amount {
   Amount { actual, ersatz }
}

#[derive(Debug, Clone)]
pub struct Asset {
   pub token: String,
   pub amount: Amount,
   quote: USD,
   kind: AssetType
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
      let hdrs = kinderize(&self.kind, &["token", "quote", "total"]);
      format!("{},{},{},{}", hdrs[0],self.kind.headers(),hdrs[1],hdrs[2]) 
   }
}

fn mk_asset(tkn: &str, amount: Amount, quote: USD, kind: AssetType) -> Asset {
   Asset { token: tkn.to_string(), amount, quote, kind }
}

fn slice2vec(ss: &[&str]) -> Vec<String> {
   let mut vec: Vec<String> = Vec::new();
   for s in ss {
      vec.push(s.to_string());
   }
   vec
}

#[derive(Debug, Clone)]
enum AssetType { FROM, TO }

impl AssetType {
   fn keys(&self) -> Vec<String> {
      match self {
         AssetType::FROM => slice2vec(&["from","amount1","virtual","quote1"]),
         AssetType::TO => slice2vec(&["to","net","blah!","quote2"])
      }
   }
   fn kind(&self) -> String {
      match self {
         AssetType::FROM => "from".to_string(),
         AssetType::TO => "to".to_string()
      }
   }
   fn headers(&self) -> String {
      let hdrs = kinderize(&self, &["actual","virtual"]);
      format!("{},{}", hdrs[0], hdrs[1])
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
      Ok(mk_asset(token, amount, quot, a))
   } else {
      Err("bad pattern match in AssetType enum for keys()".to_string())
   }
}

/// Defines the structure of an open pivot
#[derive(Debug, Clone)]
pub struct Pivot {
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
pub fn active(p: &Pivot) -> bool {
   !closed(p)
}

fn mk_lookup_f<'a>(hdrs: &'a HashMap<String, usize>, row: &'a Vec<String>)
      -> impl Fn(&'a str) -> String {
   move |key: &'a str| {
      let col = hdrs[key];
      row[col].clone()
   }
}

fn parse_header(hdrs: &HashMap<String, usize>, row: &Vec<String>)
      -> ErrStr<Header> {
   let lookf = mk_lookup_f(hdrs, row);
   let dt = lookf("opened");
   let id = parse_int(&lookf("open"))?;
   let closed = parse_int(&lookf("close"))?;
   mk_hdr(&dt, id, closed)
}

pub fn mk_pivot_2(hdrs: &HashMap<String, usize>, row: &Vec<String>)
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

pub struct Bag<T> {
    pub counts: HashMap<T, usize>,
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
}

pub async fn ingest_table() -> ErrStr<Table<usize, String, String>> {
   let piv_url = get_env("PIVOT_URL")?;
   let url = open_pivot_path(&piv_url, "btc", "eth");
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
   ingest(parse_int, parse_str, parse_str, &body, "\t")
}

pub fn actives_closeds(table: &Table<usize,String,String>)
      -> ErrStr<Partition<Pivot>> {
   let hdrs = enum_headers(cols(&table));
   let mut acts: Vec<Pivot> = Vec::new();
   let mut pass: Vec<Pivot> = Vec::new();

   for row in &table.data {
      let piv = mk_pivot_2(&hdrs, &row)?;
      if active(&piv) {
         acts.push(piv);
      } else {
         pass.push(piv);
      }
   }
   Ok((acts, pass))
}

pub fn amounts(actives: &Vec<Pivot>) -> (Bag<String>, f32, f32) {
   let mut btc: f32 = 0.0;
   let mut eth: f32 = 0.0;
   let mut bag: Bag<String> = Bag::new();

   for piv in actives {
      let tok = &piv.from.token;
      let amt: f32 = amount(&piv.from.amount);
      if tok == "BTC" { btc += amt; } else { eth += amt; }
      bag.add(tok.to_string());
   }
   (bag, btc, eth)
}

pub fn print_actives(actives: &Vec<Pivot>) {
   if actives.is_empty() {
      println!("No active pivots.");
   } else {
      println!("Active pivots\n");
      let mut x: i32 = 1;
      let mut header_printed = false;
      for piv in actives {
         if !header_printed {
            println!("ix,{}", piv.header());
            header_printed = true;
         }
         println!("{x},{}", piv.as_csv());
         x = x + 1;
      }
   }
   println!(" ");
}

pub mod functional_tests {

   use book::err_utils::ErrStr;

   use super::{ingest_table,print_actives,actives_closeds,amounts};

   pub async fn runoff() -> ErrStr<()> {

      println!("quiz01: b_table functional test.\n");

      // let's read in real open pivot data and first, put those data into a 
      // (untyped) table

      let table = ingest_table().await?;

      // We have our (unstructured) pivots tablized, now let's reify those 
      // pivots (... starting with just the opened-date data ... and now 
      // adding the rest of the header).

      // Let's also parse the FROM- and TO-assets.

      let (acts, pass) = actives_closeds(&table)?;

      let a = acts.len();
      let p = pass.len();
      println!("\nThere are {a} active pivots and {p} closed pivots.\n");
      print_actives(&acts);

      let (bag, btc, eth) = amounts(&acts);
      for (k,v) in bag.counts {
         let amt: f32 = if k == "BTC" { btc } else { eth };
         println!("There are {v} {k} open pivots, totaling {amt} {k}");
      }

      Ok(())
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   #[tokio::test]
   async fn test_ingest_table_ok() {
      let table = ingest_table().await;
      assert!(table.is_ok());
   }

   #[tokio::test]
   async fn test_actives_closeds() -> ErrStr<()> {
      let table = ingest_table().await?;
      let (acts, pass) = actives_closeds(&table)?;
      assert!(!acts.is_empty());
      assert!(!pass.is_empty());
      Ok(())
   }

   #[tokio::test]
   async fn test_amounts() -> ErrStr<()> {
      let table = ingest_table().await?;
      let (acts, _pass) = actives_closeds(&table)?;
      let (bag, btc, eth) = amounts(&acts);
      assert_eq!(2, bag.counts.len());
      assert!(eth > 0.0);
      assert!(btc > 0.0);
      Ok(())
   }
}


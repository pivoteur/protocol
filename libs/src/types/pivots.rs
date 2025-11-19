use std::collections::HashMap;

use chrono::NaiveDate;

use book::{
   csv_utils::CsvWriter,
   currency::usd::{USD,mk_usd},
   date_utils::parse_date,
   err_utils::ErrStr,
   num_utils::parse_commaless
};

use crate::{
   parsers::parse_id,
   types::{
      quotes::{Quotes,Token},
      util::{Id, CsvHeader}
   }
};

// ----- PIVOT types -------------------------------------------------------

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

pub fn parse_pivot(hdrs: &HashMap<String, usize>, row: &Vec<String>)
      -> ErrStr<Pivot> {
   let header = parse_header(hdrs, row)?;
   let from = parse_asset(AssetType::FROM, hdrs, row)?;
   let to = parse_asset(AssetType::TO, hdrs, row)?;
   Ok( Pivot { header, from, to } )
}

pub fn closed(p: &Pivot) -> bool {
   p.header.close > 0
}
pub fn active(p: &Pivot) -> bool {
   !closed(p)
}

// ----- HEADER

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

fn parse_header(hdrs: &HashMap<String, usize>, row: &Vec<String>)
      -> ErrStr<Header> {
   let dt = &row[hdrs["opened"]];
   let opn = hdrs.get("open")
                 .or(hdrs.get("pivot"))
                 .ok_or("Can't find pivot ix".to_string())?;
   let id = parse_id(&row[*opn])?;
   let closed = parse_id(&row[hdrs["close"]])?;
   mk_hdr(dt, id, closed)
}

pub fn next_close_id(pivs: &Vec<Pivot>) -> Id {
   pivs.iter().map(|p| p.header.close).max().unwrap_or(0) + 1
}

// ----- ASSETS

#[derive(Debug, Clone)]
struct Asset {
   token: Token,
   amount: Amount,
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

fn parse_asset(a: AssetType, hdrs: &HashMap<String, usize>, row: &Vec<String>)
      -> ErrStr<Asset> {
   let keys = a.keys();
   if let [tok, amt, virt, qut] = keys.as_slice() {
      let token = &row[hdrs[tok]];
      let amnt = parse_commaless(&row[hdrs[amt]])?;
      let vrt = if let Some(virt_key) = hdrs.get(virt) {
         parse_commaless(&row[*virt_key])
      } else { Ok( 0.0 ) }?;
      let quot: USD = row[hdrs[qut]].parse()?;
      let amount = mk_amt(amnt, vrt);
      Ok(mk_asset(token, amount, quot, a))
   } else {
      Err("bad pattern match in AssetType enum for keys()".to_string())
   }
}

// ----- ASSETTYPES

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

fn slice2vec(ss: &[&str]) -> Vec<String> {
   let mut vec: Vec<String> = Vec::new();
   for s in ss {
      vec.push(s.to_string());
   }
   vec
}

fn kinderize(k: &AssetType, s: &[&str]) -> Vec<String> {
   s.iter().map(|elt| format!("{}_{}", k.kind(), elt)).collect()
}

// ----- AMOUNT

#[derive(Debug, Clone)]
struct Amount {
   actual: f32,
   ersatz: f32      // 'ersatz' meaning 'virtual' as 'virtual' is reserved
}

impl CsvWriter for Amount {
   fn ncols(&self) -> usize { 2 }
   fn as_csv(&self) -> String { format!("{},{}", self.actual, self.ersatz) }
}

fn amount(a: &Amount) -> f32 { a.actual + a.ersatz }
fn mk_amt(actual: f32, ersatz: f32) -> Amount {
   Amount { actual, ersatz }
}

// ----- CLOSE PIVOTS -------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Propose {
   open_pivot: Pivot,
   close_date: NaiveDate,
   close: PropAsset
}

#[derive(Debug, Clone)]
struct PropAsset {
   close_price: USD,
   computed_amount: f32
}

#[derive(Debug, Clone)]
pub struct Close {

}

pub fn propose(q: &Quotes) -> impl Fn(&Pivot) -> ErrStr<Option<Propose>> {
   move |p: &Pivot| {
      // with the quotes for the assets, ...
      let prim = &p.from.token;
      let piv = &p.to.token;
      let prim_qt = lookup(q, prim);
      Ok(None)
   }
}


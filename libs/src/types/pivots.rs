use std::{
   collections::HashMap,
   fmt::Display
};

use chrono::{Days,NaiveDate};

use book::{
   csv_utils::CsvWriter,
   currency::usd::{USD,mk_usd},
   date_utils::parse_date,
   err_utils::ErrStr,
   num::percentage::{Percentage,mk_percentage},
   num_utils::parse_commaless,
   utils::pred
};

use crate::{
   parsers::parse_id,
   types::{
      quotes::{Quotes,lookup},
      util::{Token,Blockchain,Id,CsvHeader,Partition,Measurable,weight,size}
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
      let gain = gain_10_percent(&self.from.amount);
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

fn gain_10_percent(a: &Amount) -> f32 {
   amount(a) * 1.1
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

#[derive(Debug, Clone)]
struct AggregateHeader {
   opened: Vec<NaiveDate>,
   ids: Vec<Id>,
}

fn add_header_info(v: &Vec<Pivot>) -> AggregateHeader {
   let mut opened = Vec::new();
   let mut ids = Vec::new();
   for p in v {
      opened.push(p.header.opened.clone());
      ids.push(p.header.id);
   }
   AggregateHeader { opened, ids }
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

impl CsvHeader for AggregateHeader {
   fn header(&self) -> String { "opened,ids".to_string() }
}
impl CsvWriter for AggregateHeader {
   fn ncols(&self) -> usize { 2 }
   fn as_csv(&self) -> String {
      fn list2str<T:Display>(v: &Vec<T>) -> String {
         v.iter().map(|s| format!("{s}")).collect::<Vec<_>>().join(";")
      }
      format!("{}", list2str(&self.ids))
   }
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
   blockchain: Blockchain,
   amount: Amount,
   quote: USD,
   kind: AssetType
}

impl Measurable for Asset {
   fn sz(&self) -> f32 { amount(&self.amount) }
   fn aug(&self) -> f32 { self.sz() * self.quote.amount }
}

impl CsvWriter for Asset {
   fn ncols(&self) -> usize { 1 + 1 + self.amount.ncols() + 1 + 1}
   fn as_csv(&self) -> String {
      let total = mk_usd(self.quote.amount * amount(&self.amount));
      format!("{},{},{},{},{}",
              self.token,self.blockchain,self.amount.as_csv(),self.quote,total)
   }
}
impl CsvHeader for Asset {
   fn header(&self) -> String {
      let hdrs = kinderize(&self.kind,
                           &["token", "blockchain", "quote", "total"]);
      format!("{},{},{},{},{}",
              hdrs[0],hdrs[1],self.kind.headers(),hdrs[2],hdrs[3]) 
   }
}

fn mk_asset(tkn: &str, blk: &str, amount: Amount, quote: USD, knd: &AssetType)
      -> Asset {
   Asset { token: tkn.to_string(), 
           blockchain: blk.to_string(),
           amount, quote, kind: knd.clone() }
}

fn parse_asset(a: AssetType, hdrs: &HashMap<String, usize>, row: &Vec<String>)
      -> ErrStr<Asset> {
   let keys = a.keys();
   if let [tok, blk, amt, virt, qut] = keys.as_slice() {
      let token = &row[hdrs[tok]];
      let block = &row[hdrs[blk]];
      let amnt = parse_commaless(&row[hdrs[amt]])?;
      let vrt = if let Some(virt_key) = hdrs.get(virt) {
         parse_commaless(&row[*virt_key])
      } else { Ok( 0.0 ) }?;
      let quot: USD = row[hdrs[qut]].parse()?;
      let amount = mk_amt(amnt, vrt);
      Ok(mk_asset(token, block, amount, quot, &a))
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
         AssetType::FROM =>
            slice2vec(&["from","from_blockchain","amount1","virtual","quote1"]),
         AssetType::TO =>
            slice2vec(&["to","to_blockchain","net","blah!","quote2"])
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

// ----- GAINS -------------------------------------------------------

trait Gains {
   fn roi(&self) -> Percentage;
   fn apr(&self) -> Percentage;
}

// ----- CLOSE PIVOTS -------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Propose {
   header: AggregateHeader,
   close: Id,
   close_date: NaiveDate,
   principal: Vec<Asset>,
   pivot: Vec<PropAsset>,
   propose: PropAsset
}

pub fn pivot_amount(p: &Propose) -> ((Token, Blockchain), f32) {
   pivot_amount0(&p.pivot)
}

fn weighted_days(p: &Propose) -> ErrStr<(f32, NaiveDate)> {
   if let Some(start_date) = p.header.opened.first().cloned() {
      let days: Vec<f32> =
         p.header.opened.iter()
                        .map(|&d| ((d-start_date).num_days() + 1) as f32)
                        .collect();
      let weights: Vec<f32> =
         days.iter()
             .zip(p.principal.iter().map(Measurable::sz))
             .map(|(&a, b)| a * b)
             .collect();
      let wt: f32 = weights.iter().sum();
      let wt_days = wt / size(&p.principal);
      let ave_dt = start_date + Days::new((wt_days - 1.0) as u64);
      let duration = (p.close_date - ave_dt).num_days() as f32;
      Ok((duration, ave_dt))
   } else {
      Err("No start date for proposal".to_string())
   }
}

impl Gains for Propose {
   fn roi(&self) -> Percentage {
      let base = size(&self.principal);
      mk_percentage((self.propose.amount - base) / base)
   }
   fn apr(&self) -> Percentage {
      if let Ok((wt, _)) = weighted_days(&self) {
         mk_percentage(self.roi().of(365.0 / wt))
      } else {
         panic!("Can't get an APR for proposal")
      }
   }
}

impl CsvHeader for Propose {
   fn header(&self) -> String {
      if let Some(prince) = self.principal.first() {
         if let Some(piv) = self.pivot.first() {
            format!("{},close_id,close_date,{},gain_10_percent,{},{},roi,apr",
                    self.header.header(),
                    prince.header(),
                    piv.header(),
                    self.propose.header())
         } else {
            panic!("No pivots for proposal")
         }
      } else {
         panic!("No principal assets for proposal")
      }
   }
}
impl CsvWriter for Propose {
   fn ncols(&self) -> usize {
      if let Some(prince) = self.principal.first() {
         if let Some(piv) = self.pivot.first() {
            self.header.ncols() + 2 + prince.ncols() + 1
            + piv.ncols() + self.propose.ncols() + 2
         } else {
            panic!("No pivots for proposal")
         }
      } else {
         panic!("No principal for proposal")
      }
   }
   fn as_csv(&self) -> String {
      if let Ok((_, opnd)) = weighted_days(&self) {
         if let Some(prince) = self.principal.first() {
            let (amount, ersatz) =
               self.principal
                   .iter()
                   .fold((0.0, 0.0), |(a,e), x| {
                      let Amount { actual, ersatz } = &x.amount;
                      (a + actual, e + ersatz) });
            let pri1 = mk_asset(&prince.token, &prince.blockchain,
                                mk_amt(amount, ersatz),
                                mk_usd(weight(&self.principal)), &prince.kind);
            if let Some(piv) = self.pivot.first() {
               let piv1 = mk_prop_asset(&piv.token, &piv.blockchain,
                                        weight(&self.pivot),
                                        size(&self.pivot), &piv.kind);
               format!("{},{},{},{},{},{},{},{},{},{}", 
                       opnd,
                       self.header.as_csv(),
                       self.close,
                       self.close_date,
                       pri1.as_csv(),
                       gain_10_percent(&pri1.amount),
                       piv1.as_csv(),
                       self.propose.as_csv(),
                       self.roi(), self.apr())
            } else {
              panic!("No pivot for proposal")
            }
         } else {
            panic!("No principal for proposal")
         }
      } else {
         panic!("No open date for proposal")
      }
   }
}

fn mk_prop(open_pivots: Vec<Pivot>, c: Id, d: &NaiveDate,
           pivot: Vec<PropAsset>, propose: PropAsset) -> (Propose, Id) {
   let header = add_header_info(&open_pivots);
   let principal: Vec<Asset> =
      open_pivots.iter().map(|h| h.from.clone()).collect();
   (Propose {
      header,
      close: c,
      close_date: d.clone(),
      principal,
      pivot,
      propose }, c+1)
}

#[derive(Debug, Clone)]
struct PropAsset {
   token: Token,
   blockchain: Blockchain,
   close_price: USD,
   amount: f32,
   kind: AssetType
}

impl CsvHeader for PropAsset {
   fn header(&self) -> String {
      let preface = match self.kind {
         AssetType::FROM => "pivot",
         AssetType::TO   => "proposed"
      };
      ["token","blockchain","close_price","amount"]
         .iter()
         .map(|elt| format!("{}_{}", preface, elt))
         .collect::<Vec<_>>()
         .join(",")
   }
}
impl CsvWriter for PropAsset {
   fn ncols(&self) -> usize { 2 }
   fn as_csv(&self) -> String {
      format!("{},{},{},{}",
              self.token, self.blockchain, self.close_price, self.amount)
   }
}

impl Measurable for PropAsset {
   fn sz(&self) -> f32 { self.amount }
   fn aug(&self) -> f32 { self.sz() * self.close_price.amount }
}

fn mk_prop_asset(tkn: &str, blk: &str, c: f32, amount: f32, knd: &AssetType)
      -> PropAsset {
   PropAsset { token: tkn.to_string(), blockchain: blk.to_string(),
               close_price: mk_usd(c), amount, kind: knd.clone() }
}

fn pivot_amount0(p: &Vec<PropAsset>) -> ((Token, Blockchain), f32) {
   if let Some(fst) = p.first() {
      let tok = fst.token.clone();
      let blk = fst.blockchain.clone();
      ((tok, blk), size(p))
   } else {
      panic!("Could not find a pivot-asset for a close-recommendation!")
   }
}

pub fn propose(q: &Quotes)
      -> impl Fn((Vec<Pivot>, Id)) -> ErrStr<Option<(Propose, Id)>> {
   move |(pivots, c): (Vec<Pivot>, Id)| {
      let mut princes = Vec::new();
      let mut pivs = Vec::new();
      let mut res = Vec::new();
      for p in pivots {
         let props = trade(q, &p)?;
         let _ = props.and_then(|(frm, to)| {
            princes.push(p);
            pivs.push(frm);
            res.push(to);
            Some(1)
         });
      }
      if princes.is_empty() {
         Ok(None)
      } else {
         if let Some(result) = res.first() {
            let proposed = mk_prop_asset(&result.token, &result.blockchain,
                                         result.close_price.amount,
                                         size(&res), &AssetType::TO);
            Ok(Some(mk_prop(princes, c, &q.date, pivs, proposed)))
         } else {
            Err("No proposal to accumulate on flagged principal".to_string())
         }
      }
   }
}

fn trade(q: &Quotes, p: &Pivot) -> ErrStr<Option<(PropAsset, PropAsset)>> {
   // with the quotes for the assets, ...
   let prim = &p.from.token;
   let prim_blk = &p.from.blockchain;
   let piv = &p.to.token; 
   let piv_blk = &p.to.blockchain;
   let prim_qt = lookup(q, prim)?;
   let piv_qt = lookup(q, piv)?;
   let to_trade = amount(&p.to.amount);
   let target = gain_10_percent(&p.from.amount);
   let computed_amount = to_trade * piv_qt / prim_qt;
   Ok(pred(computed_amount > target,
           (mk_prop_asset(piv, piv_blk, piv_qt, to_trade, &AssetType::FROM),
            mk_prop_asset(prim, prim_blk, prim_qt,
                          computed_amount, &AssetType::TO))))
}

// ----- GROUPING  -------------------------------------------------------

/// Partitions open-pivots by principal asset
pub fn partition_on(tok: &str, opens: Vec<Pivot>) -> Partition<Pivot> {
   opens.into_iter().partition(|p: &Pivot| p.from.token == tok)
}


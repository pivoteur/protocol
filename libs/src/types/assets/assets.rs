use std::collections::HashMap;

use chrono::NaiveDate;

use book::{
   csv_utils::{CsvHeader,CsvWriter},
   currency::usd::{USD,mk_usd},
   err_utils::ErrStr,
   num_utils::parse_commaless,
   utils::pred
};

use super::{
   amounts::{Amount,mk_amt},
   asset_types::{AssetType,AssetType::*,kinderize}
};

use crate::types::{
   measurable::{Measurable,weight},
   coins::Coin,
   quotes::Quotes,
   proposals::prop_assets::{PropAsset,mk_prop_asset},
   util::{Token,Blockchain}
};

// ----- ASSETS

#[derive(Debug, Clone)]
pub struct Asset {
   token: Token,
   blockchain: Blockchain,
   amount: Amount,
   quote: USD,
   kind: AssetType
}

impl Asset {
   pub fn is_virt(&self) -> bool { self.amount.is_virt() }
   pub fn blockchain(&self) -> Blockchain { self.blockchain.clone() }
   pub fn token(&self) -> Token { self.token.clone() }
   pub fn committed(&self, date: &NaiveDate) -> ErrStr<Coin> {
      self.kind.committed(&self, date)
   }
}

impl Measurable for Asset {
   fn sz(&self) -> f32 { self.amount.amount() }
   fn aug(&self) -> f32 { self.quote.amount }
}

impl CsvWriter for Asset {
   fn ncols(&self) -> usize { 1 + 1 + self.amount.ncols() + 1 + 1}
   fn as_csv(&self) -> String {
      let qt = &self.quote;
      let total = mk_usd(qt.amount * self.sz());
      format!("{},{},{},{},{}",
              self.token,self.blockchain,self.amount.as_csv(),qt,total)
   }
}
impl CsvHeader for Asset {
   fn header(&self) -> String {
      let hdrs = kinderize(&self.kind, &["", "_blockchain"]);
      let idx = self.kind.ix();
      format!("{},{},{},quote{idx},val{idx}",
              hdrs[0],hdrs[1],self.kind.headers())
   }
}

pub fn mk_asset(tkn: &str, blk: &str, amount: Amount, quote: USD, 
                knd: &AssetType) -> Asset {
   Asset { token: tkn.to_string(), 
           blockchain: blk.to_string(),
           amount, quote, kind: knd.clone() }
}

pub fn parse_asset(a: AssetType, hdrs: &HashMap<String, usize>,
                   row: &Vec<String>) -> ErrStr<Asset> {
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

// ---- Worker methods -------------------------------------------------------

// for recomputing virtual pivots

pub fn recompute_assets(quotes: &Quotes, from: &Asset, to: &Asset)
      -> ErrStr<Option<(Asset, Asset)>> {
   let t2 = &to.token;
   let q2 = quotes.lookup(t2)?;
   let a2 = to.amount.amount();
   let t1 = &from.token;
   let blk = &to.blockchain;
   let q1 = quotes.lookup(t1)?;
   let tvl_now = a2 * q2;
   let a1 = from.amount.amount();
   let new_from = tvl_now / q1;
   let new_assets = if new_from < a1 {
      Some((mk_asset(t1, blk, mk_amt(0.0, new_from), mk_usd(q1), &FROM),
            mk_asset(t2, blk, to.amount.clone(), mk_usd(q2), &TO)))
   } else {
      None
   };
   Ok(new_assets)
}

// condensces a set of assets to one representative (summed) asset

pub fn coalesce(v: &Vec<Asset>) -> ErrStr<Asset> {
   if v.is_empty() { Err("No principal for proposal".to_string())
   } else { Ok(condense(v)) }
}

fn condense(v: &Vec<Asset>) -> Asset {
   let amts: Vec<Amount> = v.into_iter().map(|a| a.amount.clone()).collect();
   let prince = v.first().unwrap();
   let new_quote = mk_usd(weight(&amts));
   let amount = amts.into_iter().sum();
   mk_asset(&prince.token, &prince.blockchain, amount, new_quote, &FROM)
}

pub fn trade(q: &Quotes, from: &Asset, to: &Asset)
      -> ErrStr<Option<(PropAsset, PropAsset)>> {
   // with the quotes for the assets, ...
   let prim = &from.token;
   let prim_blk = &from.blockchain;
   let piv = &to.token;
   let piv_blk = &to.blockchain;
   let prim_qt = q.lookup(prim)?;
   let piv_qt = q.lookup(piv)?;
   let to_trade = to.sz();
   let target = gain_10_percent(from.sz());
   let computed_amount = to_trade * piv_qt / prim_qt;
   Ok(pred(computed_amount > target,
       (mk_prop_asset(piv, piv_blk, &mk_usd(piv_qt), to_trade, FROM),
        mk_prop_asset(prim, prim_blk, &mk_usd(prim_qt), computed_amount, TO))))
}

pub fn gain_10_percent(a: f32) -> f32 { a * 1.1 }

pub mod functional_tests {
   use super::*;
   use book::num::estimate::mk_estimate;

   pub fn assert_price(a: &Asset, est: f32) {
      let q1 = &a.quote;
      let qe1 = mk_estimate(q1.amount);
      let tok = &a.token;
      assert!(qe1.approximates(est * 1e03), "{tok} price ({q1}) isn't ${est}K");
   }
}


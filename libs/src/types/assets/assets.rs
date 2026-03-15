use std::collections::HashMap;

use chrono::{Days,NaiveDate};

use book::{
   csv_utils::{CsvHeader,CsvWriter},
   currency::usd::{USD,mk_usd},
   err_utils::ErrStr,
   num::percentage::{Percentage,mk_percentage},
   num_utils::parse_commaless,
   tuple_utils::Partition,
   utils::pred
};

use super::{ amounts::{Amount,mk_amt}, asset_types::{AssetType,kinderize} };

use crate::types::{
   quotes::Quotes,
   util::{Token,Blockchain,Id,Pool},
   measurable::{Measurable,weight,size},
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
}

impl Measurable for Asset {
   fn sz(&self) -> f32 { self.amount.amount() }
   fn aug(&self) -> f32 { self.quote.amount }
}

impl CsvWriter for Asset {
   fn ncols(&self) -> usize { 1 + 1 + self.amount.ncols() + 1 + 1}
   fn as_csv(&self) -> String {
      let qt = self.quote;
      let total = mk_usd(qt.amount * self.amount.amount());
      format!("{},{},{},{},{}",
              self.token,self.blockchain,self.amount.as_csv(),wt,total)
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


use chrono::NaiveDate;
use serde::{ Deserialize, Serialize };
use serde_with::{serde_as, DisplayFromStr};

use book::{
   currency::usd::{ USD, mk_usd },
   num::{ floats::comma_floats::CommaFloat, percentage::Percentage }
};

use crate::types::{ gains::Gains, measurable::Measurable, quotes::Quotes };

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PoolAssets {
    #[serde_as(as = "DisplayFromStr")]
    date: NaiveDate,
    // #[serde_as(as = "DisplayFromStr")]
    // total_invested: USD,
    #[serde_as(as = "DisplayFromStr")]
    total: USD,
    #[serde_as(as = "DisplayFromStr")]
    distributions_total: USD,
    #[serde_as(as = "DisplayFromStr")]
    roi: Percentage,
    #[serde_as(as = "DisplayFromStr")]
    apr: Percentage,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default)]
    reserve: Option<CommaFloat>
}

impl Gains for PoolAssets {
   fn roi(&self) -> Percentage { self.roi.clone() }
   fn apr(&self) -> Percentage { self.apr.clone() }
   fn gain(&self) -> f32 { panic!("gain is in two tokens") }
   fn gain_usd(&self) -> USD { self.distributions_total.clone() }
}

impl Measurable for PoolAssets {
   fn sz(&self) -> f32 { self.total.amount() }
   fn aug(&self) -> f32 { 1.0 }
}

pub fn prototype(v: &[PoolAssets]) -> Option<PoolAssets> {
   let mut assets = v.to_vec();
   assets.sort_by_key(|pa| pa.date);
   assets.last().cloned()
}

// I equate 'incept' with first pool activity, because if the funds are just
// sitting there, it's not a pivot pool, it's a puddle.
pub fn incept(pa: &[PoolAssets]) -> Option<NaiveDate> {
   prototype(pa).and_then(|p| Some(p.date.clone()))
}

impl PoolAssets {
   pub fn reserve(&self, q: &Quotes) -> USD {
      mk_usd(self.reserve.and_then(|r| {
         let res: f32 = r.into();
         match q.lookup("UNDEAD") {
            Ok(qt) => Some(qt * res),
            Err(e) => panic!("No UNDEAD quote; err: {e}")
         }
      }).unwrap_or(0.0))
   }
}

// ----- TEST -------------------------------------------------------

pub mod sample_data {
   use super::*;
   use book::{
      csv_utils::parse_items_delim,
      err_utils::ErrStr,
      file_utils::read_file
   };

   pub fn sample_btc_eth_pool_assets(offset: &str) -> ErrStr<Vec<PoolAssets>> {
      let tsv = read_file(&format!("{offset}/data/sample_btc_eth_pool.tsv"))?;
      parse_items_delim(&tsv, b'\t')
   }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use paste::paste;
   use super::*;
   use super::sample_data::sample_btc_eth_pool_assets;
   use book::{ create_testing, csv_utils::as_csv, err_utils::ErrStr };

   create_testing!("types::pools::pool_assets");

   run!("pool_assets", {
      let assets = sample_btc_eth_pool_assets("../quizzes")?;
      let top_asset = prototype(&assets).unwrap();
      println!("Pool assets:\n{}", as_csv(&[top_asset], true)?);
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use super::sample_data::sample_btc_eth_pool_assets;
   use book::err_utils::ErrStr;

   #[test] fn test_early_incept() -> ErrStr<()> {
      let assets = sample_btc_eth_pool_assets("../quizzes")?;
      let start = incept(&assets).unwrap();
      let current = assets.first().unwrap().date;
      assert!(current >= start, "{start} should preceed {current}");
      Ok(())
   }
}


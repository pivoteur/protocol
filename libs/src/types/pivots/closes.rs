use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use book::{
   currency::usd::USD,
   num::{ floats::comma_floats::CommaFloat, percentage::Percentage }
};

use crate::{
   processors::utils::{
      deserialize_optional_date,
      deserialize_semicolon_list,
      serialize_optional_date,
      serialize_semicolon_list
   },
   types::{ gains::Gains, util::Id }
};

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct ClosePivot {
    #[serde_as(as = "DisplayFromStr")]
    date: NaiveDate,
    #[serde(deserialize_with = "deserialize_optional_date")]
    #[serde(serialize_with = "serialize_optional_date")]
    opened: Option<NaiveDate>,
    #[serde(deserialize_with = "deserialize_semicolon_list")]
    #[serde(serialize_with = "serialize_semicolon_list")]
    pivot: Vec<Id>,
    close: Id,
    tx_id: String,
    from: String,
    #[serde_as(as = "DisplayFromStr")]
    from_quote: USD,
    to: String,
    #[serde_as(as = "DisplayFromStr")]
    to_quote: USD,
    #[serde_as(as = "DisplayFromStr")]
    trade: CommaFloat,
    #[serde_as(as = "DisplayFromStr")]
    vol: USD,
    gain_10_percent: f32,
    new_to_actual: f32,
    gain: f32,
    #[serde_as(as = "DisplayFromStr")]
    gain_total_usd: USD,
    #[serde_as(as = "DisplayFromStr")]
    roi: Percentage,
    #[serde_as(as = "DisplayFromStr")]
    apr: Percentage,
}

pub fn mk_close_pivot(dt: &NaiveDate, opened: Option<&NaiveDate>, opens: &[Id],
                      close: Id, tx: &str, from: &str, fqt: &USD, to: &str, 
                      tqt: &USD, trad: f32, vol: &USD, gain_10_percent: f32,
                      new_to_actual: f32, gain: f32, gain_tot: &USD,
                      r: &Percentage, a: &Percentage) -> ClosePivot {
   ClosePivot {
      date: dt.clone(),
      opened: opened.cloned(),
      pivot: opens.to_vec(),
      close,
      tx_id: tx.to_string(),
      from: from.to_string(),
      from_quote: fqt.clone(),
      to: to.to_string(),
      to_quote: tqt.clone(),
      trade: CommaFloat(trad),
      vol: vol.clone(),
      gain_10_percent,
      new_to_actual,
      gain,
      gain_total_usd: gain_tot.clone(),
      roi: r.clone(),
      apr: a.clone()
   }
}

impl Gains for ClosePivot {
   fn roi(&self) -> Percentage { self.roi.clone() }
   fn apr(&self) -> Percentage { self.apr.clone() }
   fn gain(&self) -> f32 { self.gain }
   fn gain_usd(&self) -> USD { self.gain_total_usd.clone() }
}

pub fn transform(old_row: &OldClosePivotRow, gain_10: f32) -> ClosePivot {
   let o = &old_row;
   let tr: f32 = o.trade.into();
   mk_close_pivot(&o.date, None, &o.pivot, o.close, &o.tx_id, &o.from,
                  &o.from_quote, &o.to, &o.to_quote, tr, &o.vol, gain_10,
                  o.new_to_actual, o.gain, &o.gain_total_usd, &o.roi, &o.apr)
}

/// here is the old-style close pivot
// Maps the incoming fields from the old close pivots table
#[serde_as]
#[derive(Debug, Deserialize)]
pub struct OldClosePivotRow {
    #[serde_as(as = "DisplayFromStr")]
    date: NaiveDate,
    #[serde(alias = "open")]
    #[serde(deserialize_with = "deserialize_semicolon_list")]
    pivot: Vec<Id>,
    close: Id,
    tx_id: String,
    from: String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(alias = "from quote")]
    from_quote: USD,
    to: String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(alias = "to quote")]
    to_quote: USD,
    #[serde_as(as = "DisplayFromStr")]
    trade: CommaFloat,
    #[serde_as(as = "DisplayFromStr")]
    vol: USD,
    #[serde(alias = "new to-actual")]
    new_to_actual: f32,
    gain: f32,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(alias = "gain, total $")]
    gain_total_usd: USD,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(alias = "ROI")]
    roi: Percentage,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(alias = "APR")]
    apr: Percentage,
}

impl OldClosePivotRow {
   pub fn open_pivots_ix(&self) -> Vec<Id> { self.pivot.clone() } 
}


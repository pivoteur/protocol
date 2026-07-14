use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use book::{
   currency::usd::USD,
   num::percentage::Percentage
};

#[serde_as]
#[derive(Debug, Serialize)]
pub struct ClosePivot {
    #[serde_as(as = "DisplayFromStr")]
    date: NaiveDate,
    pivot: Vec<usize>,
    close: usize,
    tx_id: String,
    from: String,
    #[serde_as(as = "DisplayFromStr")]
    from_quote: USD,
    to: String,
    #[serde_as(as = "DisplayFromStr")]
    to_quote: USD,
    trade: f32,
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

pub fn transform(old_row: &OldClosePivotRow, gain_10: f32) -> ClosePivot {
   ClosePivot {
      date: old_row.date.clone(),
      pivot: old_row.pivot.clone(),
      close: old_row.close,
      tx_id: old_row.tx_id.clone(),
      from: old_row.from.clone(),
      from_quote: old_row.from_quote.clone(),
      to: old_row.to.clone(),
      to_quote: old_row.to_quote.clone(),
      trade: old_row.trade,
      vol: old_row.vol.clone(),
      gain_10_percent: gain_10,
      new_to_actual: old_row.new_to_actual,
      gain: old_row.gain,
      gain_total_usd: old_row.gain_total_usd.clone(),
      roi: old_row.roi.clone(),
      apr: old_row.apr.clone(),
   }
}

/// here is the old-style close pivot
// Maps the incoming fields from the old close pivots table
#[serde_as]
#[derive(Debug, Deserialize)]
pub struct OldClosePivotRow {
    #[serde_as(as = "DisplayFromStr")]
    date: NaiveDate,
    #[serde(alias = "open")]
    #[serde(deserialize_with = "deserialize_csv_semicolon_list")]
    pivot: Vec<usize>,
    close: usize,
    tx_id: String,
    from: String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(alias = "from quote")]
    from_quote: USD,
    to: String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(alias = "to quote")]
    to_quote: USD,
    trade: f32,
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

fn deserialize_csv_semicolon_list<'de, D>(deserializer: D)
      -> Result<Vec<usize>, D::Error> where D: serde::Deserializer<'de> {
   // 1. Read the raw string field from the CSV record
   let s = String::deserialize(deserializer)?;
    
   // 2. Parse the semicolon-separated values into a Vec
   s.split(';')
    .map(|val| val.trim().parse::<usize>().map_err(serde::de::Error::custom))
    .collect()
}

impl OldClosePivotRow {
   pub fn open_pivots_ix(&self) -> Vec<usize> { self.pivot.clone() } 
}


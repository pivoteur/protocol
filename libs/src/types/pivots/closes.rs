use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use book::{
   currency::usd::USD,
   err_utils::ErrStr,
   num::percentage::Percentage
};

#[serde_as]
#[derive(Debug, Serialize)]
pub struct ClosePivot {
    date: String,
    pivot: String,
    close: String,
    tx_id: String,
    from: String,
    #[serde_as(as = "DisplayFromStr")]
    from_quote: USD,
    to: String,
    #[serde_as(as = "DisplayFromStr")]
    to_quote: USD,
    trade: String,
    #[serde_as(as = "DisplayFromStr")]
    vol: USD,
    gain_10_percent: f32,
    new_to_actual: String,
    gain: String,
    #[serde_as(as = "DisplayFromStr")]
    gain_total_usd: USD,
    #[serde_as(as = "DisplayFromStr")]
    roi: Percentage,
    #[serde_as(as = "DisplayFromStr")]
    apr: Percentage,
}

pub fn transform(old_row: &OldClosePivotRow, gain_10: f32) -> ClosePivot {
   ClosePivot {
      date: old_row.date,
      pivot: old_row.pivot,
      close: old_row.close,
      tx_id: old_row.tx_id,
      from: old_row.from,
      from_quote: old_row.from_quote,
      to: old_row.to,
      to_quote: old_row.to_quote,
      trade: old_row.trade,
      vol: old_row.vol,
      gain_10_percent: gain_10,
      new_to_actual: old_row.new_to_actual,
      gain: old_row.gain,
      gain_total_usd: old_row.gain_total_usd,
      roi: old_row.roi,
      apr: old_row.apr,
   }
}

/// here is the old-style close pivot
// Maps the incoming fields from the old close pivots table
#[serde_as]
#[derive(Debug, Deserialize)]
pub struct OldClosePivotRow {
    date: String,
    #[serde(alias = "open")]
    pivot: String,
    close: String,
    tx_id: String,
    from: String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(alias = "from quote")]
    from_quote: USD,
    to: String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(alias = "to quote")]
    to_quote: USD,
    trade: String,
    #[serde_as(as = "DisplayFromStr")]
    vol: USD,
    #[serde(alias = "new to-actual")]
    new_to_actual: String,
    gain: String,
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


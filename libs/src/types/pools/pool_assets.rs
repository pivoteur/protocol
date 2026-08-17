use chrono::NaiveDate;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

use book::{ currency::usd::USD, num::percentage::Percentage };

use crate::types::{ gains::Gains, measurable::Measurable };

#[serde_as]
#[derive(Debug, Deserialize)]
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
    apr: Percentage
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

// I equate 'incept' with first pool activity, because if the funds are just
// sitting there, it's not a pivot pool, it's a puddle.
pub fn incept(pa: &[PoolAssets]) -> NaiveDate {
   pa.iter().map(|p| p.date).min().unwrap()
}

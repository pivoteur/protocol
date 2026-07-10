use book::csv_utils::{ CsvWriter, CsvHeader, as_csv, list_csv };
use super::calls::Call;
use crate::types::pivots::Pivot;

pub struct CallData {
   call: Call,
   open_pivots: Vec<Pivot>
}

pub fn mk_call_data(call: Call, open_pivots: Vec<Pivot>) -> CallData {
   CallData { call, open_pivots }
}

mod sample_data {
   use super::*;
   use crate::types::{
      assets::amounts::mk_amt,
      pivots::test_data::mk_btc_usdc_piv
   };

   pub fn sample_pivot() -> Pivot {
      let piv = mk_btc_usdc_piv(78408.88, mk_amt(500.0, 0.0), 0, "https://yo");
      piv.unwrap()
   }
}

use sample_data::sample_pivot;

/// we actually just do the header for the Pivots, as the Call has its own
/// header already
impl CsvHeader for CallData {
   fn header(&self) -> String {
      sample_pivot().header()
   }
}

impl CsvWriter for CallData {
   fn ncols(&self) -> usize { sample_pivot().ncols() }
   fn as_csv(&self) -> String { 
      format!("{}\n\n{}", list_csv(&self.open_pivots),
              as_csv(&[&self.call]).unwrap())
   }
}

impl CallData {
   pub fn virtual_pivots(&self) -> Vec<Pivot> {
      let mut pivs = self.open_pivots.clone();
      pivs.retain(|p| p.is_virtual());
      pivs
   }
   pub fn tvl(&self) -> USD {

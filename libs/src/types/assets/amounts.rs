use std::{ ops::Add, iter::Sum };

use book::csv_utils::CsvWriter;

use crate::types::measurable::Measurable;

// ----- AMOUNT

#[derive(Debug, Clone)]
pub struct Amount {
   actual: f32,
   ersatz: f32      // 'ersatz' meaning 'virtual' as 'virtual' is reserved
}

// ----- CONSTRUCTOR ---------------------------------------------------------

pub fn mk_amt(actual: f32, ersatz: f32) -> Amount {
   Amount { actual, ersatz }
}

impl Amount {
   pub fn is_virt(&self) -> bool { self.ersatz > 0.0 }
   pub fn amount(&self) -> f32 { self.actual + self.ersatz }
}

impl Measurable for Amount {
   fn sz(&self) -> f32 { self.amount() }
   fn aug(&self) -> f32 { 1.0 }
}

impl CsvWriter for Amount {
   fn ncols(&self) -> usize { 2 }
   fn as_csv(&self) -> String { format!("{},{}", self.actual, self.ersatz) }
}

impl Add for Amount {
   type Output = Self;
   fn add(self, other: Self) -> Self {
      Amount { actual: self.actual + other.actual,
               ersatz: self.ersatz + other.ersatz }
   }
}

impl Sum for Amount {
   fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
      iter.fold(Amount { actual: 0.0, ersatz: 0.0 }, |a,b| a + b)
   }
}


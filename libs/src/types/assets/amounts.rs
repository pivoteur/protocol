use book::csv_utils::CsvWriter;

// ----- AMOUNT

#[derive(Debug, Clone)]
pub struct Amount {
   actual: f32,
   ersatz: f32      // 'ersatz' meaning 'virtual' as 'virtual' is reserved
}

impl Amount {
   pub fn is_virt(&self) -> bool { self.ersatz > 0.0 }
   pub fn amount(&self) -> f32 { self.actual + self.ersatz }
}

impl CsvWriter for Amount {
   fn ncols(&self) -> usize { 2 }
   fn as_csv(&self) -> String { format!("{},{}", self.actual, self.ersatz) }
}

pub fn mk_amt(actual: f32, ersatz: f32) -> Amount {
   Amount { actual, ersatz }
}


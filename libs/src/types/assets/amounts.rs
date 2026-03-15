use book::csv_utils::CsvWriter;

// ----- AMOUNT

#[derive(Debug, Clone)]
pub struct Amount {
   actual: f32,
   ersatz: f32      // 'ersatz' meaning 'virtual' as 'virtual' is reserved
}

fn is_virt2(a: &Amount) -> bool { a.ersatz > 0.0 }

impl CsvWriter for Amount {
   fn ncols(&self) -> usize { 2 }
   fn as_csv(&self) -> String { format!("{},{}", self.actual, self.ersatz) }
}

pub fn amount(a: &Amount) -> f32 { a.actual + a.ersatz }
pub fn mk_amt(actual: f32, ersatz: f32) -> Amount {
   Amount { actual, ersatz }
}


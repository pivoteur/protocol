use serde::Serialize;

use crate::types::measurable::Measurable;

#[derive(Debug, Clone, Serialize)]
pub struct Committed {
   #[serde(rename = "virtual")]
   virtual_amt: f32,
   #[serde(rename = "actual")]
   actual_amt: f32
}

pub fn mk_committed(virtual_amt: f32, actual_amt: f32) -> Committed {
   Committed { virtual_amt, actual_amt }
}

impl Measurable for Committed {
   fn sz(&self) -> f32 { self.virtual_amt + self.actual_amt }
   fn aug(&self) -> f32 { 1.0 }
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, err_utils::{ ErrStr, err_or } };

   use serde_json;

   create_testing!("types::tokens::committed");

   run!("serialize_committed", {
      let commit = mk_committed(2.5, 3.6);
      let json = err_or(serde_json::to_string_pretty(&commit),
                        "Could not serialize commit as JSON")?;
      println!("{commit:?} as JSON: {json}");
   });
}


use serde::{Serialize, ser::SerializeStruct, Serializer};

use super::{ allocation_builder::AllocationBuilder, committed::Committed };

use crate::types::{ measurable::Measurable, util::Token };

use book::string_utils::s;

#[derive(Debug, Clone)]
pub struct Allocation {
   token: Token,
   total: f32,
   committed: Committed
}

pub fn mk_allocation(tok: &str, total: f32, committed: Committed)
      -> Allocation {
   Allocation { token: s(tok), total, committed }
}

impl Measurable for Allocation {
   fn sz(&self) -> f32 { self.total }
   fn aug(&self) -> f32 { 1.0 }
}

impl Allocation {
   pub fn available(&self) -> f32 { self.sz() - self.committed.sz() }
   pub fn builder() -> AllocationBuilder { AllocationBuilder::new() }
   pub fn key(&self) -> Token { self.token.clone() }
   pub fn committed(&self) -> Committed { self.committed.clone() }
}

impl Serialize for Allocation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Tell Serde you are writing a struct with 4 fields: 3 real + 1 derived
        let mut state = serializer.serialize_struct("Allocation", 4)?;

        state.serialize_field("token", &self.token)?;
        state.serialize_field("total", &self.total)?;
        state.serialize_field("committed", &self.committed)?;

        // Compute and inject the virtual field on the fly
        state.serialize_field("available", &self.available())?;
        
        state.end()
    }
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod test_data {
   use super::*;
   use crate::types::tokens::allocations::committed::mk_committed;
   use book::err_utils::ErrStr;

   pub fn sample_allocation(tok: &str, virt: f32, act: f32)
         -> ErrStr<Allocation> {
      let commit = mk_committed(virt, act);
      Allocation::builder()
         .token(tok).total(commit.sz() * 1.2).committed(commit).build()
   }
   pub fn sample_allocation0() -> ErrStr<Allocation> {
      sample_allocation("BTC", 1.2, 9.7)
   }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::test_data::sample_allocation0;
   use paste::paste;
   use book::{ create_testing, err_utils::{ ErrStr, err_or } };
   use serde_json;

   create_testing!("types::tokens::allocations");

   run!("serialize_allocation", {
      let alloc = sample_allocation0()?;
      let json = err_or(serde_json::to_string_pretty(&alloc),
                        "Unable to serialize Allocation to JSON")?;
      println!("{alloc:?} as JSON:\n\n{json}");
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::test_data::sample_allocation0;
   use book::{ err_utils::ErrStr, num::estimate::mk_estimate };

   #[test] fn test_available_assets_for_allocation() -> ErrStr<()> {
      let alloc = sample_allocation0()?;
      let estimate = mk_estimate(2.18);
      let avail = alloc.available();
      assert!(estimate.approximates(avail), "Available amount is {avail}");
      Ok(())
   }
}

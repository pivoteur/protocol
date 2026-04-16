use book::{
   err_utils::ErrStr,
};

pub fn usage() -> String {
   println!("\n$ ./aurora <protocol> <date> [min=1000]

Creates virtual pivots based upon available assets.

where
* <protocol> is the protocol, e.g. PIVOT
* <date> to check availability, e.g.: $LE_DATE
* [min] minimum pivot amount (default $1000.00)
");
   "Needs arguments <protocol> <date>, optionally [min=1000]".to_string()
}

// ----- TESTS -------------------------------------------------------

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use book::{date_utils::yesterday, utils::get_args};

   pub async fn runoff_with_args() -> ErrStr<()> {
      let _args = get_args();
      Ok(())
   }

   pub async fn runoff() -> ErrStr<usize> {
      let _yday = yesterday();
      println!("{}", usage());
      Ok(1)
   }

   #[cfg(test)]
   mod tests {
      use super::*;

      #[test]
      fn test_usage() {
         let use_this_pitch = usage();
         assert!(!use_this_pitch.is_empty());
      }
   }      
}


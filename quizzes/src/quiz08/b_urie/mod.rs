// code goes in here

// including functional and unit tests

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use book::{ err_utils::ErrStr, test_utils::preamble };

   pub fn runoff() -> ErrStr<usize> {
      preamble("quiz08::b_urie");

      // code

      Ok(1)
   }
}


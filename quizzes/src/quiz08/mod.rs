pub mod a_table;
pub mod b_urie;

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use book::{ err_utils::ErrStr, test_utils::preamble };
   use super::{
      a_table::functional_tests::runoff as a,
      b_urie::functional_tests::runoff as b
   };

   pub fn runoff() -> ErrStr<usize> {
      preamble("quiz08");
      let n1 = a()?;
      let n2 = b()?;
      Ok(n1+n2)
   }
}


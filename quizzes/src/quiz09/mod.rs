
pub mod a_itr;

pub mod functional_tests {
   use super::a_itr::functional_tests::runoff as a;

   use book::err_utils::ErrStr;

   pub fn runoff() -> ErrStr<usize> {
      println!("\nquiz09 functional tests\n");
      a()
   }
}

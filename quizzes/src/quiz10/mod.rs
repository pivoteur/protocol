
pub mod a_files;

pub mod functional_tests {
   use super::a_files::functional_tests::runoff as a;

   use book::err_utils::ErrStr;

   pub fn runoff() -> ErrStr<usize> {
      println!("\nquiz10 functional tests\n");
      a()
   }
}

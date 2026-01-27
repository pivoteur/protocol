pub mod a_quotes;
pub mod b_compute_close;

pub mod functional_tests {

   use book::err_utils::ErrStr;

   use super::{
      a_quotes::functional_tests::runoff as a,
      b_compute_close::functional_tests::runoff as b
   };

   pub async fn runoff() -> ErrStr<()> {
      println!("\nquiz02 functional tests\n");
      a().await.and(b().await)
   }
}


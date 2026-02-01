pub mod a_partition;

pub mod functional_tests {

   use book::err_utils::ErrStr;

   use super::{
      a_partition::functional_tests::runoff as a,
      // b_compute_close::functional_tests::runoff as b
   };

   pub async fn runoff() -> ErrStr<()> {
      println!("\nquiz03 functional tests\n");
      a().await.and(Ok(()))  // b().await)
   }
}


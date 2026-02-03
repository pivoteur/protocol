pub mod a_partition;
pub mod b_aggregate;

pub mod functional_tests {

   use book::err_utils::ErrStr;

   use super::{
      a_partition::functional_tests::runoff as a,
      b_aggregate::functional_tests::runoff as b
   };

   pub async fn runoff() -> ErrStr<usize> {
      println!("\nquiz03 functional tests\n");
      let n1 = a().await?;
      let n2 = b().await?;
      Ok(n1 + n2)
   }
}


pub mod a_read;
pub mod b_table;

pub mod functional_tests {

   use book::err_utils::ErrStr;

   use super::{
      a_read::functional_tests::runoff as a,
      b_table::functional_tests::runoff as b
   };

   pub async fn runoff() -> ErrStr<usize> {
      println!("\nquiz01 functional tests\n");
      let n1 = a().await?;
      let n2 = b().await?;
      Ok(n1 + n2)
   }
}


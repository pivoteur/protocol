pub mod a_quotes;
pub mod b_compute_close;

pub mod functional_tests {

   use book::err_utils::ErrStr;

   use super::{
      a_quotes::functional_tests::runoff as a,
      b_compute_close::functional_tests::runoff as b
   };

   pub async fn runoff() -> ErrStr<usize> {
      println!("\nquiz02 functional tests\n");
      let n1 = a().await?;
      let n2 = b().await?;
      Ok(n1 + n2)
   }
}


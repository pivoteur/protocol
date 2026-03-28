pub mod a_ssets;
pub mod b_virtual;

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {

   use super::{
      a_ssets::functional_tests::runoff as a,
      b_virtual::functional_tests::runoff as b
   };
   use book::err_utils::ErrStr;

   pub async fn runoff() -> ErrStr<usize> {
      let n1 = a().await?;
      let n2 = b().await?;
      Ok(n1 + n2)
   }
}


pub mod a_ssets;
pub mod b_virtual;
pub mod c_open;

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {

   use super::{
      a_ssets::functional_tests::runoff as a,
      b_virtual::functional_tests::runoff as b,
      c_open::functional_tests::runoff as c
   };
   use book::err_utils::ErrStr;

   pub async fn runoff() -> ErrStr<usize> {
      let n1 = a().await?;
      let n2 = b().await?;
      let n3 = c().await?;
      Ok(n1 + n2 + n3)
   }
}


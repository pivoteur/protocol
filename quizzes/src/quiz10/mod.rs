
pub mod a_files;
pub mod b_pools;
pub mod c_local_pools;

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::{
      a_files::functional_tests::runoff as a,
      b_pools::functional_tests::runoff as b,
      c_local_pools::functional_tests::runoff as c
   };

   use book::err_utils::ErrStr;

   pub async fn runoff() -> ErrStr<usize> {
      println!("\nquiz10 functional tests\n");
      let n1 = a()?;
      let n2 = b().await?;
      let n3 = c().await?;
      Ok(n1 + n2 + n3)
   }
}

pub mod a_pool_table;
pub mod b_pools;
pub mod c_assets;

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {

   use super::{
      a_pool_table::functional_tests::runoff as a,
      b_pools::functional_tests::runoff as b,
      c_assets::functional_tests::runoff as c
   };
   use book::err_utils::ErrStr;

   pub async fn runoff() -> ErrStr<usize> {
      let n1 = a().await?;
      let n2 = b().await?;
      let n3 = c().await?;
      Ok(n1 + n2 + n3)
   }
}

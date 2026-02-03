pub mod a_pool_table;
// pub mod b_pools;

pub mod functional_tests {

   use super::a_pool_table::functional_tests::runoff as a;
   use book::err_utils::ErrStr;

   pub async fn runoff() -> ErrStr<usize> {
      let n1 = a().await?;
      Ok(n1)
   }
}

pub mod a_assets;

pub mod functional_tests {

   use super::{
      a_assets::functional_tests::runoff as a
   };
   use book::err_utils::ErrStr;

   pub async fn runoff() -> ErrStr<usize> {
      let n1 = a().await?;
      Ok(n1)
   }
}

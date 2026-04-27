pub mod a_git_json;

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use book::{ err_utils::ErrStr, test_utils::preamble };

   use super::a_git_json::functional_tests::runoff as a;

   fn module() -> String { "quiz04".to_string() }
   pub async fn runoff() -> ErrStr<usize> {
      preamble(&module());
      let n1 = a().await?;
      Ok(n1)
   }
}


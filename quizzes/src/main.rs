use book::{ err_utils::ErrStr, test_utils::{preamble,report_test_results} };

#[cfg(not(tarpaulin_include))]
mod test_infrastructure {
   use futures::Future;

   use super::*;

   use book::test_utils::test_result;

   use quizzes::{
      quiz01::functional_tests::runoff as a,
      quiz02::functional_tests::runoff as b,
      quiz03::functional_tests::runoff as c,
      quiz04::functional_tests::runoff as d,
      quiz05::functional_tests::runoff as e,
      quiz06::functional_tests::runoff as f,
      quiz07::functional_tests::runoff as g,
      quiz08::functional_tests::runoff as h,
      quiz09::functional_tests::runoff as i,
      quiz10::functional_tests::runoff as j
   };

   fn two_digits(n: usize) -> String {
      format!("{}{n}", if n < 10 { "0" } else { "" })
   }

   pub fn test_names() -> Vec<String> {
      [1,2,3,4,5,6,7,8,9,10]
           .iter().map(|n| format!("quiz{}", two_digits(*n))).collect()
   }

   async fn run_testa<F: Future<Output = ErrStr<usize>>>(name: &str, test: F)
         -> ErrStr<usize> {
      let res = test.await;
      test_result(name, res)
   }

   pub async fn tests() -> Vec<ErrStr<usize>> {
      vec![run_testa("quiz01",a()).await,
           run_testa("quiz02",b()).await,
           run_testa("quiz03",c()).await,
           run_testa("quiz04",d()).await,
           run_testa("quiz05",e()).await,
           run_testa("quiz06",f()).await,
           run_testa("quiz07",g()).await,
           test_result("quiz08",h()),
           test_result("quiz09",i()),
           run_testa("quiz10", j()).await]
   }
}

use test_infrastructure::{tests, test_names};

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> ErrStr<()> {
   preamble("quizzes");
   let res = tests().await;
   match report_test_results("quizzes", &test_names(), res) {
      Ok(_) => Ok(()),
      Err(x) => Err(x)
   }
}


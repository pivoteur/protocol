use futures::Future;

use book::{
   err_utils::ErrStr,
   test_utils::{report_test_results,test_result}
};

use quizzes::{
   quiz01::functional_tests::runoff as a,
   quiz02::functional_tests::runoff as b,
   quiz03::functional_tests::runoff as c,
   quiz06::functional_tests::runoff as f,
   quiz09::functional_tests::runoff as i,
   quiz10::functional_tests::runoff as j
};

fn two_digits(n: usize) -> String {
   format!("{}{n}", if n < 10 { "0" } else { "" })
}

fn test_names() -> Vec<String> {
   [1,2,3,6,9,10].iter().map(|n| format!("quiz{}", two_digits(*n))).collect()
}

async fn run_testa<F: Future<Output = ErrStr<usize>>>(name: &str, test: F)
      -> ErrStr<usize> {
   let res = test.await;
   test_result(name, res)
}

async fn tests() -> Vec<ErrStr<usize>> {
   vec![run_testa("quiz01",a()).await,
        run_testa("quiz02",b()).await,
        run_testa("quiz03",c()).await,
        run_testa("quiz06",f()).await,
        test_result("quiz09",i()),
        run_testa("quiz10", j()).await]
}

#[tokio::main]
async fn main() -> ErrStr<()> {
   println!("quizzes functional tests\n");
   let res = tests().await;
   match report_test_results(test_names(), res) {
      Ok(_) => Ok(()), Err(x) => Err(x)
   }
}


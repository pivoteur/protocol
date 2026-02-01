use book::{
   err_utils::ErrStr,
   test_utils::{collate_results,mk_tests}
};

use quizzes::{
   quiz01::functional_tests::runoff as a,
   quiz02::functional_tests::runoff as b,
   quiz03::functional_tests::runoff as c,
   quiz09::functional_tests::runoff as i
};

fn test_names() -> Vec<String> {
   [1,2,3,9].iter().map(|n| format!("quiz0{n}")).collect()
}
async fn tests() -> Vec<ErrStr<()>> { 
   [a().await, b().await, c().await, i()].to_vec()
}

#[tokio::main]
async fn main() -> ErrStr<()> {
   println!("\nquizzes functional tests\n");
   collate_results("quizzes", &mk_tests(&test_names().join(" "), &tests()))
}


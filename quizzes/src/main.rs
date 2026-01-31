use book::{
   err_utils::ErrStr,
   test_utils::collate_results
};

use quizzes::{
   quiz01::functional_tests::runoff as a,
   quiz02::functional_tests::runoff as b,
   quiz09::functional_tests::runoff as i
};

fn test_names() -> Vec<String> {
   [1,2,9].iter().map(|n| format!("quiz0{n}")).collect()
}
async fn tests() -> Vec<ErrStr<()>> { [a().await, b().await, i()].to_vec() }

#[tokio::main]
async fn main() -> ErrStr<()> {
   println!("\nquizzes functional tests\n");
   collate_results(&tests().await, &test_names())
}


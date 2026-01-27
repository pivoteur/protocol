use book::err_utils::ErrStr;

use quizzes::{
   quiz01::functional_tests::runoff as a,
   quiz02::functional_tests::runoff as b
};

#[tokio::main]
async fn main() -> ErrStr<()> {
   println!("\nquizzes functional tests\n");
   a().await.and(b().await)
}


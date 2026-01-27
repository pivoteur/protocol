use book::err_utils::ErrStr;

use quizzes::{
   quiz01::functional_tests::runoff as a
};

#[tokio::main]
async fn main() -> ErrStr<()> {
   a().await.and(Ok(()))
}


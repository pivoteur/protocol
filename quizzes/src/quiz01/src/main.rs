use book::err_utils::ErrStr;

use quizzes::quiz01::functional_tests::runoff;

#[tokio::main]
async fn main() -> ErrStr<()>  { runoff().await }


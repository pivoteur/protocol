use book::err_utils::ErrStr;

use quizzes::quiz04::functional_tests::runoff_with_args;

#[tokio::main]
async fn main() -> ErrStr<()> { runoff_with_args().await }


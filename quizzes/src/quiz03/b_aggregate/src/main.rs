use book::err_utils::ErrStr;

use quizzes::quiz03::b_aggregate::functional_tests::runoff_get_args;

#[tokio::main]
async fn main() -> ErrStr<()> { runoff_get_args().await }


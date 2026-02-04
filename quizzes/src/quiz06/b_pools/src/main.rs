use quizzes::quiz06::b_pools::functional_tests::runoff_get_args;

use book::err_utils::ErrStr;

#[tokio::main]
async fn main() -> ErrStr<()> { runoff_get_args().await }


use quizzes::quiz07::a_ssets::functional_tests::runoff_with_args;
use book::err_utils::ErrStr;

#[tokio::main]
async fn main() -> ErrStr<()> { runoff_with_args().await }


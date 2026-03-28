use quizzes::quiz07::a_ssets::functional_tests::runoff_with_args;
use book::err_utils::ErrStr;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> ErrStr<()> { runoff_with_args().await }


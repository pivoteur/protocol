use book::err_utils::ErrStr;

use quizzes::quiz10::c_local_pools::functional_tests::runoff_with_args;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> ErrStr<()> { runoff_with_args().await }


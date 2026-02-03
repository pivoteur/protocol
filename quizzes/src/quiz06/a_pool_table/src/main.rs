use book::err_utils::ErrStr;

use quizzes::quiz06::a_pool_table::functional_tests::runoff_get_args;

#[tokio::main]
async fn main() -> ErrStr<()> { let _ = runoff_get_args().await?; Ok(()) }


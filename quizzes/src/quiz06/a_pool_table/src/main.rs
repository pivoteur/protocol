use book::err_utils::ErrStr;

use quizzes::quiz06::a_pool_table::runoff_get_args;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> ErrStr<()> { let _ = runoff_get_args().await?; Ok(()) }


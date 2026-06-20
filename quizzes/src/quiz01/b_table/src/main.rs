use book::err_utils::ErrStr;

use quizzes::quiz01::b_table::runoff_no_args;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> ErrStr<()> { let _ = runoff_no_args().await }

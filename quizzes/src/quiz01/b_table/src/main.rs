use book::err_utils::ErrStr;

use quizzes::quiz01::b_table::runoff;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> ErrStr<()> { let _ = runoff().await?; Ok(()) }

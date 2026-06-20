use book::err_utils::ErrStr;

use quizzes::quiz01::a_read::runoff_no_args as r;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> ErrStr<()> { r().await }


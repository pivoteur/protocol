use book::err_utils::ErrStr;

use quizzes::quiz02::a_quotes::runoff_no_args as r;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> ErrStr<()> { r().await }


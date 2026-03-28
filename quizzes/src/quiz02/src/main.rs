use book::err_utils::ErrStr;

use quizzes::quiz02::functional_tests::runoff;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> ErrStr<()>  { runoff().await }


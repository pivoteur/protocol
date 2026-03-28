use quizzes::quiz07::b_virtual::functional_tests::runoff_with_args as r;

use book::err_utils::ErrStr;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> ErrStr<()> { r().await }


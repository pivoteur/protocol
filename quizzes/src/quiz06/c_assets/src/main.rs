use quizzes::quiz06::c_assets::functional_tests::runoff_get_args;

use book::err_utils::ErrStr;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> ErrStr<()> { runoff_get_args().await }


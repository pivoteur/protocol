use book::err_utils::ErrStr;

use quizzes::quiz06::functional_tests::runoff as q;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> ErrStr<()> { let _ = q().await?; Ok(()) }


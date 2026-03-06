use book::err_utils::ErrStr;

use quizzes::quiz07::functional_tests::runoff as q;

#[tokio::main]
async fn main() -> ErrStr<()> { let _ = q().await?; Ok(()) }


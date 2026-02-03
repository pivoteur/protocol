use book::err_utils::ErrStr;

use quizzes::quiz06::functional_tests::runoff as q;

#[tokio::main]
async fn main() -> ErrStr<()> { let _ = runoff().await?; Ok(()) }


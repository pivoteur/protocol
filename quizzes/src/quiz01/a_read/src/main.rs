use book::err_utils::ErrStr;

use quizzes::quiz01::a_read::functional_tests::runoff;

#[tokio::main]
async fn main() -> ErrStr<()> { let _ = runoff().await?; Ok(()) }


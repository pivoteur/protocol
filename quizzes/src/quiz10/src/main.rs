use book::err_utils::ErrStr;
use quizzes::quiz10::functional_tests::runoff;

#[tokio::main]
async fn main() -> ErrStr<()> { let _ = runoff().await?; Ok(()) }

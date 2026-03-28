use book::err_utils::ErrStr;
use quizzes::quiz10::functional_tests::runoff;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> ErrStr<()> { let _ = runoff().await?; Ok(()) }

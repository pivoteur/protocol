use book::err_utils::ErrStr;
use quizzes::quiz10::d_quotes::runoff_with_args as r;

#[tokio::main] async fn main() -> ErrStr<()> { r().await }

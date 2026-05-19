use book::err_utils::ErrStr;
use quizzes::quiz08::b_wyrd::runoff_with_args;

#[tokio::main] async fn main() -> ErrStr<()> { runoff_with_args().await }

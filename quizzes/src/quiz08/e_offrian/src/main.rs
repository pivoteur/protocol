use quizzes::quiz08::e_offrian::runoff_with_args as r;

use book::err_utils::ErrStr;

#[tokio::main] async fn main() -> ErrStr<()> { r().await }


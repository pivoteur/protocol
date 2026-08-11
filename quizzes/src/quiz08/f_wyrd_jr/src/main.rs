use book::err_utils::ErrStr;
use quizzes::quiz08::f_wyrd_jr::runoff_with_args as r;

#[tokio::main]
async fn main() -> ErrStr<()> { r().await }

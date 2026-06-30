use book::err_utils::ErrStr;
use quizzes::quiz05::b_dusk_min::runoff_with_args as a;

#[tokio::main] async fn main() -> ErrStr<()> { a().await }


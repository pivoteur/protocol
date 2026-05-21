use book::err_utils::ErrStr;
use quizzes::quiz11::a_reinvested::runoff_with_args;

#[tokio::main]
async fn main() -> ErrStr<()> {
    runoff_with_args().await
}

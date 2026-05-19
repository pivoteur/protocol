use book::err_utils::ErrStr;
use quizzes::quiz07::d_health::runoff_with_args;

#[tokio::main]
async fn main() -> ErrStr<()> {
   runoff_with_args().await
}

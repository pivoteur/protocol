use book::err_utils::ErrStr;
use quizzes::quiz05::b_dusk_min::functional_tests::runoff_with_args as a;

#[tokio::main]
async fn main() -> ErrStr<()> {
    a().await
}

use book::err_utils::ErrStr;

use quizzes::quiz05::a_assets::functional_tests::runoff_with_args as a;

#[tokio::main]
async fn main() -> ErrStr<()> { a().await }


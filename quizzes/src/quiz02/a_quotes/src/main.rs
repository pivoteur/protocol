use book::err_utils::ErrStr;

use quizzes::quiz02::a_quotes::functional_tests::runoff;

#[tokio::main]
async fn main() -> ErrStr<()> { runoff().await }


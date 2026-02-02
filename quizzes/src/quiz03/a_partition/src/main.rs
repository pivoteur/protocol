use book::err_utils::ErrStr;

use quizzes::quiz03::a_partition::functional_tests::runoff_get_args as r;

#[tokio::main]
async fn main() -> ErrStr<()> { r().await }


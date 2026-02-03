use book::err_utils::ErrStr;

use quizzes::quiz02::b_compute_close::functional_tests::runoff_get_args;

#[tokio::main]
async fn main() -> ErrStr<()> { runoff_get_args().await }


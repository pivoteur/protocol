use book::err_utils::ErrStr;

use quizzes::quiz11::functional_tests::runoff as r;

#[tokio::main]
async fn main() -> ErrStr<()> {
   let _ = r().await?;
   Ok(())
}


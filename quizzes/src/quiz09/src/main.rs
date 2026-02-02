use book::err_utils::ErrStr;

use quizzes::quiz09::functional_tests::runoff;

fn main() -> ErrStr<()> { let _ = runoff()?; Ok(()) }


use book::err_utils::ErrStr;

use quizzes::quiz09::functional_tests::runoff;

#[cfg(not(tarpaulin_include))]
fn main() -> ErrStr<()> { let _ = runoff()?; Ok(()) }


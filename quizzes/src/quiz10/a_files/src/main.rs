use book::err_utils::ErrStr;

use quizzes::quiz10::a_files::functional_tests::runoff_get_args;

#[cfg(not(tarpaulin_include))]
fn main() -> ErrStr<()> { runoff_get_args() }

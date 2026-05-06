use book::err_utils::ErrStr;

use quizzes::quiz09::a_itr::runoff_get_args;

#[cfg(not(tarpaulin_include))]
fn main() -> ErrStr<()> { runoff_get_args() }

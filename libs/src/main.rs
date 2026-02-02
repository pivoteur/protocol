// we run the functional tests for the libs here

use book::{
   err_utils::ErrStr,
   test_utils::{collate_results,mk_tests,mk_async,mk_sync}
};

use libs::{
   git::functional_tests::runoff as g,
   tables::functional_tests::runoff as t
};

#[tokio::main]
async fn main() -> ErrStr<()> {
   let tests = vec![mk_async(g()), mk_sync(t)];
   let _ = collate_results("libs", mk_tests("git tables", tests))?;
   Ok(())
}


// we run the functional tests for the libs here

use book::{
   err_utils::ErrStr,
   test_utils::{collate_results,mk_tests,mk_async,mk_sync}
};

use libs::{
   fetchers::functional_tests::runoff as f,
   git::functional_tests::runoff as g,
   tables::functional_tests::runoff as t,
   types::pivots::functional_tests::runoff as p
};

#[tokio::main]
async fn main() -> ErrStr<()> {
   let tests = vec![mk_async(f()), mk_async(g()), mk_sync(t), mk_sync(p)];
   let _ = collate_results("libs",
              &mut mk_tests("fetchers git tables types::pivot", tests))?;
   Ok(())
}


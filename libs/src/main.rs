// we run the functional tests for the libs here

use book::{
   err_utils::ErrStr,
   test_utils::{collate_results,mk_tests,mk_async,mk_sync}
};

use libs::{
   fetchers::functional_tests::runoff as f,
   git::functional_tests::runoff as g,
   tables::functional_tests::runoff as t,
   types::pivots::functional_tests::runoff as tp,
   types::proposals::proposes::functional_tests::runoff as tpp,
   types::assets::assets::functional_tests::runoff as taa
};

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> ErrStr<()> {
   let tests = vec![mk_async(f()), mk_async(g()), mk_sync(t),
                    mk_sync(tp), mk_sync(tpp), mk_sync(taa)];
   let test_names =
      vec!["fetchers git tables types::pivot types::proposals::proposes",
           "types::assets::assets"];
   let _ = collate_results("libs",
              &mut mk_tests(&test_names.join(" "), tests))?;
   Ok(())
}


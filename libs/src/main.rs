// we run the functional tests for the libs here

use book::{
   err_utils::ErrStr,
   string_utils::words,
   test_utils::{preamble,run_test,mk_sync,test_result,report_test_results}
};

use libs::{
   fetchers::functional_tests::runoff as f,
   paths::functional_tests::runoff as p,
   processors::functional_tests::runoff as pft,
   tables::functional_tests::runoff as t,
   types::pivots::functional_tests::runoff as tp,
   types::proposals::proposes::functional_tests::runoff as tpp,
   types::assets::assets::functional_tests::runoff as taa
};

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> ErrStr<()> {
   preamble("libs");
   preamble("fetchers");
   let run_fetchers = f().await;
   let a = test_result("fetchers", run_fetchers);
   let run_processors = pft().await;
   let b = test_result("processors", run_processors);
   let c = run_test("paths", &mut mk_sync(p));
   let d = run_test("tables", &mut mk_sync(t));
   let e = run_test("types::pivot", &mut mk_sync(tp));
   let f = run_test("types::proposals::proposes", &mut mk_sync(tpp));
   let g = run_test("types::assets::assets", &mut mk_sync(taa));
   let t1 = "fetchers processors paths tables types::pivot";
   let t2 = "types::proposals::proposes types::assets::assets";
   let tests = words(&format!("{t1} {t2}"));
   let _ur_mom = report_test_results("libs", &tests, [a,b,c,d,e,f,g].to_vec())?;
   Ok(())
}


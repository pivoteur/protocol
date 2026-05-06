// we run the functional tests for the libs here

use book::{
   err_utils::ErrStr,
   string_utils::{s,words},
   test_utils::{preamble,run_test,mk_sync,test_result,report_test_results}
};

use libs::{
   collections::assets::functional_tests::runoff as ca,
   fetchers::functional_tests::runoff as f,
   paths::functional_tests::runoff as p,
   processors::functional_tests::runoff as pft,
   tables::functional_tests::runoff as t,
   types::{
      pivots::functional_tests::runoff as tp,
      proposals::proposes::functional_tests::runoff as tpp,
      assets::assets::functional_tests::runoff as taa,
      coins::functional_tests::runoff as tc,
      comps::functional_tests::runoff as tco,
   }
};

async fn run_async<F: Future<Output = ErrStr<usize>>>(test: &str, f: F)
      -> ErrStr<(String, ErrStr<usize>)> {
   preamble(test);
   let runnoft = f.await;
   let a = test_result(test, runnoft);
   Ok((s(test), a))
}

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> ErrStr<()> {
   preamble("libs");
   let (an, a) = run_async("fetchers", f()).await?;
   let (bn, b) = run_async("processors", pft()).await?;
   let c = run_test("paths", &mut mk_sync(p));
   let d = run_test("tables", &mut mk_sync(t));
   let e = run_test("types::pivot", &mut mk_sync(tp));
   let f = run_test("types::proposals::proposes", &mut mk_sync(tpp));
   let g = run_test("types::assets::assets", &mut mk_sync(taa));
   let (hn, h) = run_async("types::coins", tc()).await?;
   let (im, i) = run_async("types::comps", tco()).await?;
   let (jn, j) = run_async("collections::assets", ca()).await?;

   let t1 = "paths tables types::pivot";
   let t2 = "types::proposals::proposes types::assets::assets";
   let tests = words(&format!("{an} {bn} {t1} {t2} {hn} {im} {jn}"));
   let _ur_mom =
      report_test_results("libs", &tests, [a,b,c,d,e,f,g,h,i,j].to_vec())?;
   Ok(())
}


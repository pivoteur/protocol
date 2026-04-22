// we run the functional tests for the libs here

use book::{
   err_utils::ErrStr,
   string_utils::plural,
   test_utils::{run_test,mk_sync,Thunk}
};

use libs::{
   fetchers::functional_tests::runoff as f,
   paths::functional_tests::runoff as p,
   tables::functional_tests::runoff as t,
   types::pivots::functional_tests::runoff as tp,
   types::proposals::proposes::functional_tests::runoff as tpp,
   types::assets::assets::functional_tests::runoff as taa
};

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> ErrStr<()> {
   preamble("book");
   preamble("fetchers");
   let run_fetchers = f().await?;
   let a = rpt(run_fetchers, "fetchers")?;
   let b = 0; // git module deprecated
   let c = report_test("paths", &mut mk_sync(p))?;
   let d = report_test("tables", &mut mk_sync(t))?;
   let e = report_test("types::pivot", &mut mk_sync(tp))?;
   let f = report_test("types::proposals::proposes", &mut mk_sync(tpp))?;
   let g = report_test("types::assets::assets", &mut mk_sync(taa))?;
   let _ur_mom = rpt(a+b+c+d+e+f+g, "book")?;
   Ok(())
}

fn preamble(test: &str) { println!("{test} functional tests\n"); }
fn report_test(test: &str, t: &mut Thunk) -> ErrStr<usize> {
   let len = run_test(test, t)?;
   rpt(len, test)
}

fn rpt(len: usize, test: &str) -> ErrStr<usize> {
   let desig = if len == 1 { "The" } else { "All" };
   println!("\n{desig} {} passed.\n",
            plural(len, &format!("{test} functional test")));
   Ok(len)
}


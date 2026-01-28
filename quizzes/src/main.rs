use book::{
   err_utils::ErrStr,
   string_utils::plural
};

use quizzes::{
   quiz01::functional_tests::runoff as a,
   quiz02::functional_tests::runoff as b,
   quiz09::functional_tests::runoff as i
};

#[tokio::main]
async fn main() -> ErrStr<()> {
   println!("\nquizzes functional tests\n");
   fn okey() -> ErrStr<()> { Ok(()) }
   let (c,d,e,f,g,h) = (okey(), okey(), okey(), okey(), okey(), okey());
   let test_results = [a().await, b().await, c, d, e, f, g, h, i()];
   let res: Vec<usize> = 
      test_results.iter()
                  .enumerate()
                  .filter_map(|(a,b)| if b.is_ok() { None } else { Some(a+1) })
                  .collect();
   let n = test_results.len();
   if res.is_empty() {
      let tests = plural(n, "functional test");
      println!("\nAll {tests} passed.");
      Ok(())
   } else {
      let fails = plural(res.len(), &format!("/{n} functional test"));
      Err(format!("{fails} failed for quizzes {res:?}"))
   }
}


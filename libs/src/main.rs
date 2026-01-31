// we run the functional tests for the libs here

use book::{
   err_utils::ErrStr,
   string_utils::words,
   test_utils::collate_results
};

use libs::{
   git::functional_tests::runoff as g,
   tables::functional_tests::runoff as t
};

fn test_names() -> Vec<String> { words("git table") }
async fn tests() -> Vec<ErrStr<()>> {
   // let names = test_names();
   let res = vec![g().await, t()];
   // names.into_iter().zip(res.into_iter()).collect()
   res
}

#[tokio::main]
async fn main() -> ErrStr<()> {
   collate_results(&tests().await, &test_names())
}


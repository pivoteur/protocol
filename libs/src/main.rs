// we run the functional tests for the libs here

use book::{
   err_utils::ErrStr,
   string_utils::{plural,words},
   utils::pred
};

use libs::{
   git::functional_tests::runoff as g,
   tables::functional_tests::runoff as t
};

fn tests() -> Vec<String> {
   words("git tables")
   // vec!["git", "tables"].into_iter().map(to_string).collect()
}

#[tokio::main]
async fn main() -> ErrStr<()> {
   let res = [g().await];
   let len = res.len();
   if res.iter().all(Result::is_ok) {
      println!("\nAll {} passed.", plural(len, "functional test"));
      Ok(())
   } else {
      failures(&res, len)
   }
}

fn failures(res: &[ErrStr<()>], len: usize) -> ErrStr<()> {
   let fs: Vec<String> =
      res.iter()
         .enumerate()
         .filter_map(|(n,r)| pred(!r.is_ok(), tests()[n].clone()))
         .collect();
   let many = plural(fs.len(), &format!("/{len} functional test"));
   println!("The following {} FAILED!:\n\t{}", many, fs.join("\n\t"));
   Err(format!("{} FAILED!", many))
}


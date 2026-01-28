use std::process::{Command,Stdio};

use book::{
   err_utils::ErrStr,
   file_utils::subdirs,
   string_utils::plural,
   tuple_utils::Partition
};

fn app_name() -> String { "itr".to_string() }

fn build_dapps(dir: &str) -> Partition<String> {
   let dirs = subdirs(dir);
   let mut successes = Vec::new();
   let mut failures = Vec::new();

   for dir in dirs {
      let status =
         Command::new("cargo")
                .arg("build")
                .current_dir(&dir)
                .stderr(Stdio::null())
                .status()
                .expect("Did not exit build-process!");
      let dir_name =
          dir.file_name()
             .expect(&format!("no dir named {dir:?}"))
             .to_string_lossy()
             .to_string();
      if status.success() {
         successes.push(dir_name);
      } else {
         failures.push(dir_name);
      }
   }
   (successes,failures)
}

fn report_build_results(p: Partition<String>) -> ErrStr<()> {
   println!("Integration test results\n");
   let (s, f) = &p;
   let flen = f.len();
   let tot = s.len() + flen;
   report_dirs("Successful dapp builds", "ok", s, tot);
   report_dirs("Build failures", "FAILED", f, tot);
   if flen > 0 {
      Err(format!("{}!", plural(flen, "build failure")))
   } else { Ok(()) }
}

fn report_dirs(hdr: &str, kind: &str, dirs: &[String], total: usize) {
   let nix = dirs.is_empty();
   let prefix = if nix { "No " } else { "" };
   println!("{prefix}{hdr}\n");
   if !nix {
      for dir in dirs { println!("{dir}:...{kind}"); }
      println!("\ntotal: {}/{total}\n", dirs.len());
   }
}

fn usage() -> String {
   println!("Usage:

	$ {} <dir>

where <dir> 
is the directory where cargo build will be executed in each dapp-directory.
", app_name());
   "dapps <dir> argument required".to_string()
}

pub mod functional_tests {
   use super::*;

   use book::utils::get_args;

   fn version() -> String { "1.00".to_string() }
   fn print_heading() { println!("{}, version: {}\n", app_name(), version()); }

   pub fn runoff_get_args() -> ErrStr<()> {
      let args = get_args();
      do_it(args.first().as_deref().map(|s| s.as_str()))
   }

   fn do_it(mb_dir: Option<&str>) -> ErrStr<()> {
      print_heading();
      let dir = mb_dir.ok_or_else(|| usage())?;
      let res = build_dapps(dir);
      report_build_results(res)
   }

   fn runoff1() -> ErrStr<()> {
      println!("\nquiz09: a_itr build successes\n");
      do_it(Some("data/sample_dapps"))
   }

   fn runoff2() -> ErrStr<()> {
      println!("\nquiz09: a_itr build FAILURE!\n");
      match do_it(Some("data/sample_broken_dapp")) {
         Ok(()) => Err("No build failures detected!".to_string()),
         Err(_) => Ok(())
      }
   }

   pub fn runoff() -> ErrStr<()> { runoff1().and(runoff2()) }
}

#[cfg(test)]
mod tests {
   use super::*;

   fn good_dir() -> String { "data/sample_dapps".to_string() }
   fn bad_dir() -> String { "data/sample_broken_dapp".to_string() }

   #[test]
   fn test_build_dapps() {
      let (a, b) = build_dapps(&good_dir());
      assert!(a.len() > 0);
      assert!(b.is_empty());
   }

   #[test]
   fn fail_build_dapps() {
      let (_a, b) = build_dapps(&bad_dir());
      assert!(!b.is_empty());
   }
}


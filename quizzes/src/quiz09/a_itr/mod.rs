use std::process::{Command,Stdio};

use clap::{ Parser, CommandFactory };

use book::{
   parse_args_add_banner,
   cli_utils::add_banner,
   err_utils::ErrStr,
   file_utils::subdirs,
   string_utils::plural,
   tuple_utils::Partition,
};

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
   match flen {
      0 => Ok(()),
      _ => Err(format!("{}!", plural(flen, "build failure")))
   }
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

/// Runs integration tests by building all dapps of the protocol
#[derive(Debug, Parser)]
#[command(name = "itr")]
#[command(version = "1.02")]
struct Args {
   /// directory in which the protocol dapps reside
   dir: String
}

pub fn runoff_get_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   println!("{}", Args::command().render_version());
   build_dapps_and_report(&args.dir)
}

fn build_dapps_and_report(dir: &str) -> ErrStr<()> {
   let res = build_dapps(dir);
   report_build_results(res)
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod test_data {
   pub fn good_dir() -> &'static str { "data/sample_dapps" }
   pub fn bad_dir() -> &'static str { "data/sample_broken_dapp" }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use super::test_data::*;
   use paste::paste;
   use book::create_testing;

   create_testing!("quiz09::a_itr");

   run!("build_dapps_success", " (successes)",
        build_dapps_and_report(good_dir()));
   run!("build_dapps_failure", " (build FAILURE!)",
        build_dapps_and_report(bad_dir()));
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use super::test_data::*;

   #[test] fn test_build_dapps() {
      let (a, b) = build_dapps(&good_dir());
      assert!(a.len() > 0);
      assert!(b.is_empty());
   }

   #[test] fn fail_build_dapps() {
      let (_a, b) = build_dapps(&bad_dir());
      assert!(!b.is_empty());
   }
}


use book::{
   err_utils::ErrStr,
   file_utils::{dirs_files,file_names},
   utils::get_args
};

fn app_name() -> String { "files".to_string() }
fn version() -> String { "1.00".to_string() }

fn print_files(dir: &str) {
   let (_dirs, files) = dirs_files(&dir);
   let names = file_names(&files);
   names.iter().for_each(|name| { println!("{name}"); });
}

fn usage() -> ErrStr<()> {
   println!("Usage:

$ {} <dir>

Lists the files in directory <dir>", app_name());
   Err("Missing <dir> argument".to_string())
}

pub mod functional_tests {

   use super::*;

   pub fn runoff() -> ErrStr<usize> {
      println!("\nquiz10: a_files functional test\n");
      let libs_dir = format!("../libs/src");
      print_files(&libs_dir);
      Ok(1)
   }

   pub fn runoff_get_args() -> ErrStr<()> {
      println!("\n{}, version: {}\n", app_name(), version());
      let args = get_args();
      if let Some(dir) = args.first() {
         print_files(&dir);
         Ok(())
      } else {
         usage()
      }
   }
}


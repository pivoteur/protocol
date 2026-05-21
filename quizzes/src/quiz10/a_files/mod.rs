use book::{
   err_utils::ErrStr,
   file_utils::{dirs_files,file_names},
   utils::get_args
};

fn app_name() -> String { "files".to_string() }
fn version() -> String { "1.00".to_string() }

fn files_as_str(dir: &str) -> String {
   let (_dirs, files) = dirs_files(&dir);
   let names = file_names(&files);
   names.join("\n")
}

fn print_files(dir: &str) { println!("{}", files_as_str(dir)); }

fn usage() -> ErrStr<()> {
   println!("Usage:

$ {} <dir>

Lists the files in directory <dir>", app_name());
   Err("Missing <dir> argument".to_string())
}

#[cfg(not(tarpaulin_include))]
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

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {

   use super::*;
   use paste::paste;
   use book::create_testing;

   create_testing!("quiz10::a_files");

   run_with!("files_as_str", "../libs/src", files_as_str);
}


use clap::Parser;

use book::{
   parse_args_add_banner,
   cli_utils::add_banner,
   err_utils::ErrStr,
   file_utils::{dirs_files,file_names}
};

fn files_as_str(dir: &str) -> String {
   let (_dirs, files) = dirs_files(&dir);
   let names = file_names(&files);
   names.join("\n")
}

fn print_files(dir: &str) -> ErrStr<()> {
   println!("{}", files_as_str(dir));
   Ok(())
}

/// Lists the files in directory
#[derive(Debug, Parser)]
struct Args {
   /// directory to list files
   dir: String
}

#[cfg(not(tarpaulin_include))]
pub fn runoff_get_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   print_files(&args.dir)
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


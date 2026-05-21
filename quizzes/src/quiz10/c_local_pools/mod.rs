use book:: {
   err_utils::ErrStr,
   date_utils::parse_date,
   utils::get_args
};
use pools_impl::print_pools_as_js;
fn app_name() -> String { "pools".to_string() }
fn version() -> String { "2.02".to_string() }

fn usage() -> ErrStr<()> {
   println!("
Usage:
      
$ {} <auth> <date>
      
Given <auth> access and <date>, {} prints a Javascript object of pool assets.
", app_name(), app_name());
   Err("Need <auth> and <date> arguments".to_string()) 
}     

#[cfg(not(tarpaulin_include))]
pub async fn runoff_with_args() -> ErrStr<()> {
   println!("\n// created by: {}, version: {}\n", app_name(), version());

   let args = get_args();
   if let [auth, dt] = args.as_slice() {
      let date = parse_date(&dt)?;
      print_pools_as_js(&auth, date).await
   } else {
      usage()
   }
}

mod pools_impl {
   use chrono::NaiveDate;
   
   use book::{
      err_utils::ErrStr,
      file_utils::{dirs_files,file_names},
      list_utils::filter_map_or,
      string_utils::str2strf,
      utils::get_env
   };
   use libs::{ paths::pivot_pool_from_file, types::util::Pool };

   pub async fn print_pools_as_js(auth: &str, date: NaiveDate) -> ErrStr<()> {
      let a = pools(auth).await?;
      let js = to_js(date, a);
      println!("{js}\n");
      Ok(())
   }
   
   pub async fn pools(auth: &str) -> ErrStr<Vec<Pool>> {
      let aut = auth.to_uppercase();
      let path = get_env(&format!("{aut}_DATA_DIR"))?;
      let open_dir = format!("{path}/pivots/open/raw");
      let (_dirs, opens_filebufs) = dirs_files(&open_dir);
      let opens = file_names(&opens_filebufs);
      filter_map_or(str2strf(&pivot_pool_from_file), opens)
   }
   
   fn to_js(dt: NaiveDate, pools: Vec<Pool>) -> String {
      fn pool2pool(p: Pool) -> String {
         let (a, b) = p;
         format!("[ '{}', '{}' ]", a.to_uppercase(), b.to_uppercase())
      }
      let assets: Vec<String> = pools.into_iter().map(pool2pool).collect();
      format!(" 
const poolAssets = {{
   generated: '{dt}',
   assets: [
      {}
      ]
}};
",  assets.join(",\n      "))
   }
}

// ----- TESTS -------------------------------------------------------

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod tests {
   use super::*;
   use std::collections::HashSet;
   use book::date_utils::today;
   use libs::types::util:: { mk_pool, Pool };
   use pools_impl::pools;

   #[tokio::test] async fn fail_print_pools_as_js() {
      assert!(print_pools_as_js("asdf", today()).await.is_err());
   }

   #[tokio::test] async fn test_print_pools_as_js_ok() {
      assert!(print_pools_as_js("pivot", today()).await.is_ok());
   }

   #[tokio::test] async fn test_pools_has_btc_eth() -> ErrStr<()> {
      let pivot_pools = pools("pivot").await?;
      let pp: HashSet<Pool> = pivot_pools.into_iter().collect();
      assert!(pp.contains(&mk_pool("btc", "eth")));
      Ok(())
   }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::pools_impl::print_pools_as_js;
   use paste::paste;
   use book::{
      create_testing,
      date_utils::today,
      err_utils::ErrStr,
      utils::now
   };

   create_testing!("quiz10::c_local_pools");

   run!("print_pools_as_js", {
      let date = today();
      let _ = now(print_pools_as_js("pivot", date));
   });
}


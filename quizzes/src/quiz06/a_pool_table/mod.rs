use book::{err_utils::ErrStr, string_utils::s, utils::{get_env,get_args}};
use libs::{
   fetchers::assets::pool::fetch_assets,
   reports::print_table,
   types::{aliases::aliases,pools::{Pool,mk_pool}}
};

fn version() -> String { s("0.01") }
fn app_name() -> String { s("dawn") }

fn usage() -> ErrStr<()> { println!("
Shows the assets of the selected pivot pool.

Usage:

	$ {} <auth> <primary> <pivot>

where

* <auth> is protocol authenticator
* <primary> is the pool's primary asset
* <pivot> is the pool's pivot asset
", app_name());
   Err(s("Need <root URL> <primary> <pivot> arguments!"))
}

// as this function only calls a library function, it's not testable:
// it's infrastructure.
async fn fetch_pool_assets(auth: &str, pool: &Pool, debug: bool)
      -> ErrStr<usize> {
   let aliases = aliases();
   let root = get_env(&format!("{}_URL", auth.to_uppercase()))?;
   let og = fetch_assets(&root, pool, &aliases, debug).await?;
   print_table(&format!("{} assets", pool.pool_name()), &[og]);
   Ok(1)
}

pub async fn runoff_get_args() -> ErrStr<()> {
   println!("\n{}, version: {}\n", app_name(), version());
   let (debug, args) = get_args();
   if let [auth, prim, piv] = args.as_slice() {
      let pool = mk_pool(&prim, &piv);
      let _ = fetch_pool_assets(auth, &pool, debug).await?;
      Ok(())
   } else {
      usage()
   }
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {

   use super::*;
   use paste::paste;
   use book::{ create_testing, utils::now };

   create_testing!("quiz06::a_pool_table", "", true);

   run!("fetch_pool_assets",
        now(fetch_pool_assets("pivot", &mk_pool("btc", "eth"), true)));
}


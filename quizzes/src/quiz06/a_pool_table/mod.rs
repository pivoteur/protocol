use book::err_utils::ErrStr;

fn version() -> String { "0.01".to_string() }
fn app_name() -> String { "dawn".to_string() }

fn usage() -> ErrStr<()> { println!("
Shows the assets of the selected pivot pool.

Usage:

	$ {} <auth> <primary> <pivot>

where

* <auth> is protocol authenticator
* <primary> is the pool's primary asset
* <pivot> is the pool's pivot asset
", app_name());
   Err("Need <root URL> <primary> <pivot> arguments!".to_string())
}

pub mod functional_tests {

   use super::*;

   use book::utils::{get_args,get_env};

   use libs::{
      fetchers::fetch_assets,
      reports::{print_table,header},
      types::aliases::aliases
   };

   async fn fetch_pool_assets(auth: &str, prim: &str, piv: &str)
         -> ErrStr<usize> {
      let aliases = aliases();
      let root = get_env(&format!("{}_URL", auth.to_uppercase()))?;
      let og = fetch_assets(&root, &prim, &piv, &aliases).await?;
      print_table(&format!("{} assets", header(&prim, &piv)), &[og]);
      Ok(1)
   }

   pub async fn runoff_get_args() -> ErrStr<()> {
      println!("\n{}, version: {}\n", app_name(), version());
      let args = get_args();
      if let [auth, prim, piv] = args.as_slice() {
         let _ = fetch_pool_assets(auth, prim, piv).await?;
         Ok(())
      } else {
         usage()
      }
   }

   pub async fn runoff() -> ErrStr<usize> {
      fetch_pool_assets("pivot", "btc", "eth").await
   }
}

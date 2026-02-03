use book::err_utils::ErrStr;

fn version() -> String { "0.01".to_string() }
fn app_name() -> String { "dawn".to_string() }

fn usage() -> ErrStr<()> { println!("
Shows the assets of the selected pivot pool.

Usage:

	$ {} <root URL> <primary> <pivot>

where

* <root URL> is repository REST endpoint
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
      reports::{print_table,header}
   };

   pub async fn runoff_get_args() -> ErrStr<()> {
      println!("\n{}, version: {}\n", app_name(), version());
      let args = get_args();
      if let [root, prim, piv] = args.as_slice() {
         let og = fetch_assets(&root, &prim, &piv).await?;
         print_table(&format!("{} assets", header(&prim, &piv)), &[og]);
         Ok(())
      } else {
         usage()
      }
   }

   pub async fn runoff() -> ErrStr<usize> {
      let piv = get_env("PIVOT_URL")?;
      let pool = fetch_assets(&piv, "BTC", "ETH").await?;
      print_table("BTC+ETH assets", &[pool]);
      Ok(1)
   }
}

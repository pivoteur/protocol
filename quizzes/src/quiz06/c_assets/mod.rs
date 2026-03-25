use book::{ date_utils::parse_date, err_utils::ErrStr };

fn version() -> String { "1.03".to_string() }
fn app_name() -> String { "assets".to_string() }

fn usage() -> ErrStr<()> { println!("
Shows the assets of the selected pivot pool.

Usage:

	$ {} <auth> <date>

where

* <auth> is protocol authenticator
* <date> the date of the assets-value, e.g.: $LE_DATE
", app_name());
   Err("Need <root URL> <primary> <pivot> arguments!".to_string())
}

async fn compute_assets(root_url: &str, date: $str)
      -> ErrStr<HashMap<String,USD>> {
   let dt = parse_date(date)?;
   let qts = fetch_quotes(&dt).await?;
   
}

pub mod functional_tests {

   use super::*;

   use book::utils::{get_args,get_env};

   use libs::{
      fetchers::{fetch_wallets,fetch_quotes},
      reports::{print_table,header},
      types::aliases::aliases
   };

   async fn fetch_assets(auth: &str) -> ErrStr<usize> {
        
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

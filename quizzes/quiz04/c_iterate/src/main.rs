use book::{
   date_utils::parse_date,
   utils::{get_args,get_env},
   err_utils::ErrStr
};

use libs::{
   git::fetch_pool_names,
   processors::process_pools
};

fn version() -> String { "1.04".to_string() }

#[tokio::main]
async fn main() -> ErrStr<()> {
   if let [ath, dt] = get_args().as_slice() {
      let auth = ath.to_uppercase();
      let date = parse_date(dt)?;
      let root = get_env(&format!("{auth}_URL"))?;
      let pools = fetch_pool_names(&auth).await?;
      println!("hound, version {}\n", version());
      process_pools(&root, &pools, date).await?;
      Ok(())
   } else {
      usage()
   }
}

fn usage() -> ErrStr<()> {
   println!("Usage:

$ cargo run <auth> <date>

where:

 * <auth> authorization token name to git repository
 * <date> Today's date
");
   Err("Requires <auth> <date> arguments".to_string())
}

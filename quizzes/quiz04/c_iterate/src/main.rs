use book::{
   utils::get_args,
   err_utils::ErrStr
};

use libs::{
   processors::process_pools,
   reports::report_proposes
};

fn app_name() -> String { "phound".to_string() }
fn version() -> String { "2.01".to_string() }

#[tokio::main]
async fn main() -> ErrStr<()> {
   if let [ath, dt] = get_args().as_slice() {
      let (proposals, no_closes) = process_pools(&ath, &dt).await?;
      println!("{}, version {}\n", app_name(), version());
      report_proposes(&proposals, &no_closes);
      Ok(())
   } else {
      usage()
   }
}

fn usage() -> ErrStr<()> {
   println!("Usage:

$ {} <auth> <date>

where:

 * <auth> authorization token name to git repository
 * <date> Today's date
", app_name());
   Err(format!("{} requires <auth> <date> arguments", app_name()))
}

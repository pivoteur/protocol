use book::{
   utils::get_args,
   err_utils::ErrStr
};

use libs::{
   processors::process_pools,
   reports::report_proposes
};

fn version() -> String { "1.05".to_string() }

#[tokio::main]
async fn main() -> ErrStr<()> {
   if let [ath, dt] = get_args().as_slice() {
      let (proposals, no_closes) = process_pools(&ath, &dt).await?;
      println!("hound, version {}\n", version());
      report_proposes(&proposals, &no_closes);
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

use book::{
   utils::get_args,
   err_utils::ErrStr
};

use libs::git::fetch_pool_names;

#[tokio::main]
async fn main() -> ErrStr<()> {
   if let Some(auth) = get_args().first() {
      let pools = fetch_pool_names(&auth).await?;
      println!("Pivot pools:\n");
      for (ix, pool) in pools.iter().enumerate() {
         println!("{}. {pool:?}", ix+1);
      }
      Ok(())
   } else {
      Err("No auth token".to_string())
   }
}

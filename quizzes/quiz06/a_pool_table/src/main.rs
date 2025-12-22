use book::{
   err_utils::ErrStr,
   utils::get_args
};

use libs::{
   fetchers::fetch_assets,
   reports::print_table
};

#[tokio::main]
async fn main() -> ErrStr<()> {
   if let Some(root) = get_args().first() {
      let og = fetch_assets(&root, "BTC", "ETH").await?;
      print_table("BTC+ETH assets", &[og]);
      Ok(())
   } else {
      Err("Needs <root_url> argument".to_string())
   }
}

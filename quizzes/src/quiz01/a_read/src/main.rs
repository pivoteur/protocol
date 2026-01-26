use book::err_utils::ErrStr;

use quiz01::a_read::reader;

#[tokio::main]
async fn main() -> ErrStr<()> {
   let body = reader().await?;
   println!("I got {body}");
   Ok(())
}

use book::{err_utils::ErrStr, rest_utils::read_rest};

pub async fn reader() -> ErrStr<String> {
   let piv = "https://raw.githubusercontent.com/pivoteur/pivoteur.github.io";
   let opens = "refs/heads/main/data/pivots/open/raw";
   let btc_eth = format!("{}/{}/btc-eth.tsv", piv, opens);
   read_rest(&btc_eth).await
}

pub mod functional_tests {

   use book::err_utils::ErrStr;

   use super::reader;

   pub async fn runoff() -> ErrStr<usize> {
      println!("quiz01: a_read functional test.\n");

      let body = reader().await?;
      println!("I got {body}");
      Ok(1)
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   #[tokio::test]
   async fn test_reader_ok() {
      let body = reader().await;
      assert!(body.is_ok());
   }

   #[tokio::test]
   async fn test_reader_body() -> ErrStr<()> {
      let body = reader().await?;
      assert!(!body.is_empty());
      Ok(())
   }
}

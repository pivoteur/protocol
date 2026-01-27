use book::{ err_utils::ErrStr, rest_utils::read_rest };

pub async fn reader() -> ErrStr<String> {
   let piv = "https://raw.githubusercontent.com/pivoteur/pivoteur.github.io";
   let opens = "refs/heads/main/data/pivots/open/raw";
   let btc_eth = format!("{}/{}/btc-eth.tsv", piv, opens);
   read_rest(&btc_eth).await
}

pub mod functional_tests {

   use book::err_utils::ErrStr;

   use super::reader;

   pub async fn runoff() -> ErrStr<()> {
      println!("a_read functional test.\n");

      let body = reader().await?;
      println!("I got {body}");
      Ok(())
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   #[tokio::test]
   async fn test_reader() -> ErrStr<()> {
      let body = reader().await?;
      assert!(!body.is_empty());
      Ok(())
   }
}

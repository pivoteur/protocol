use book::{err_utils::ErrStr, rest_utils::read_rest};

pub async fn reader() -> ErrStr<String> {
   let piv = "https://raw.githubusercontent.com/pivoteur/pivoteur.github.io";
   let opens = "refs/heads/main/data/pivots/open/raw";
   let btc_eth = format!("{}/{}/btc-eth.tsv", piv, opens);
   read_rest(&btc_eth).await
}

// ----- TESTS -------------------------------------------------------
#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use book:: {
      err_utils::ErrStr,
      create_testing,
      utils::now
   };
   use paste::paste;
   use super::reader;


   create_testing!("quiz01::a_read");

   run!("reader", {
      println!("quiz01: a_read functional test.\n");
      let body = now(reader());
      println!("I got {:?}", body);
   }); 
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

use book::{err_utils::ErrStr, rest_utils::read_rest};

async fn reader() -> ErrStr<String> {
   let piv = "https://raw.githubusercontent.com/pivoteur/pivoteur.github.io";
   let opens = "refs/heads/main/data/pivots/open/raw";
   let btc_eth = format!("{}/{}/btc-eth.tsv", piv, opens);
   read_rest(&btc_eth).await
}

pub async fn runoff_no_args() -> ErrStr<()> {
   let body = reader().await?;
   println!("I got {body}");
   Ok(())
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, utils::now };

   create_testing!("quiz01::a_read");

   run!("a_read", now(runoff_no_args()) );
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
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

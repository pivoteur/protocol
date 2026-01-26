use book::{ err_utils::ErrStr, rest_utils::read_rest };

pub async fn reader() -> ErrStr<String> {
   let piv = "https://raw.githubusercontent.com/pivoteur/pivoteur.github.io";
   let opens = "refs/heads/main/data/pivots/open/raw";
   let btc_eth = format!("{}/{}/btc-eth.tsv", piv, opens);
   read_rest(&btc_eth).await
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

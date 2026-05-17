use book::err_utils::ErrStr;
use crate::{ paths::tsv_url, tables::{IxTable,index_table} };
use super::utils::fetch_lines;

// ----- WALLETS ----------------------------------------------------

pub async fn fetch_wallets(root_url: &str) -> ErrStr<IxTable> {
   let url = tsv_url(root_url, "wallets");
   let lines = fetch_lines(&url).await?;
   index_table(lines)
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{
      create_testing,
      csv_utils::CsvWriter,
      utils::now
   };
   use crate::fetchers::test_helpers::test_functions::marshall;

   create_testing!("fetchers::wallets");

   run!("fetch_wallets", {
      let (root_url, _aliases) = marshall()?;
      let wallets = now(fetch_wallets(&root_url))?;
      println!("The wallets for {root_url} are:\n\n{}\n",
               wallets.as_csv());
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use crate::fetchers::test_helpers::test_functions::marshall;

   #[tokio::test]
   async fn test_fetch_wallets_ok() -> ErrStr<()> {
      let (root_url, _aliases) = marshall()?;
      let ans = fetch_wallets(&root_url).await;
      assert!(ans.is_ok());
      Ok(())
   }

   #[tokio::test]
   async fn test_fetch_wallets_table_data() -> ErrStr<()> {
      let (root_url, _aliases) = marshall()?;
      let ans = fetch_wallets(&root_url).await?;
      assert!(!ans.data.is_empty());
      Ok(())
   }
}

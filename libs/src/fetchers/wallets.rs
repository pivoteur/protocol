use reqwest::header::{ HeaderMap, HeaderValue };

use book::{
   err_utils::{ErrStr,err_or},
   utils::get_env
};
use crate::{
   paths::tsv_url,
   tables::{IxTable,index_table},
   types::{
      tokens::moralis::{Tokens,Blockchain}
   }
};
use super::utils::fetch_lines;

// ----- WALLETS ----------------------------------------------------

/// This fetches alles the wallets from github (manually updated for now)
pub async fn fetch_wallets(root_url: &str) -> ErrStr<IxTable> {
   let url = tsv_url(root_url, "wallets");
   let lines = fetch_lines(&url).await?;
   index_table(lines)
}

// Function to fetch native balance (e.g., ETH, MATIC)
pub async fn fetch_wallet_balances(auth: &str, ch: Blockchain)
      -> ErrStr<Tokens> {

/*
This function models the following cURL command:

curl --request GET \
  --url 'https://deep-index.moralis.io/api/v2.2/wallets/{address}/tokens?chain=eth' \
  --header 'X-API-Key: test'
*/

    let chain = ch.blockchain();
    let aut = auth.to_uppercase();
    let address = get_env(&format!("{aut}_WALLET_ADDY"))?;
    let api_key = get_env(&format!("{aut}_MORALIS_API_KEY"))?;

    let url0 = "https://deep-index.moralis.io/api/v2.2/wallets";
    let url = format!("{url0}/{address}/tokens?chain={chain}");
    let mut headers = HeaderMap::new();
    let api_hdr = err_or(HeaderValue::from_str(&api_key),
                         "Cannot insert MORALIS_API_KEY into header")?;
    headers.insert("X-API-Key", api_hdr);

    let client = reqwest::Client::new();
    let res = 
       err_or(client.get(&url).headers(headers).send().await,
              "Failed to send reqwest to moralis.io")?;
    let toks: Tokens =
       err_or(res.json().await, "Cannot convert response from JSON")?;
    Ok(toks)
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{
      create_testing,
      csv_utils::{CsvWriter,CsvHeader,enumerate_csv},
      currency::usd::USD,
      string_utils::words,
      types::filters::{ Container, mk_whitelist, Sieve },
      utils::now
   };
   use crate::{
      fetchers::test_helpers::test_functions::marshall,
      types::{ measurable::tvl, tokens::moralis::Blockchain::AVALANCHE }
   };

   create_testing!("fetchers::wallets");

   run!("fetch_wallets", " (fetch all wallets from github)", {
      let (root_url, _aliases) = marshall()?;
      let wallets = now(fetch_wallets(&root_url))?;
      println!("The wallets for {root_url} are:\n\n{}\n",
               wallets.as_csv());
   });

   run!("fetch_wallet_balances", " (no filter)", {
      now(iter_chains_on(Sieve))
   });

   run!("fetch_wallet_balances_whitelisted", {
// removed: 0xb97ef9ef8734c71904d8002f8b6bc66dd9c48a6e
      let whitelist = mk_whitelist(words("
Protocol
AVAX 0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee
Tokens
UNDEAD 0x5a3534720a4f29fa0dc53ce474db88973a95f65c
WETH 0x49d5c2bdffac6ce2bfdb6640f4f80f226bc10bab
USDt 0x9702230a8ea53601f5cd2dc00fdbc13d4df4a8c7
USDC 0xb97ef9ef8734c71904d8002f8b6bc66dd9c48a6e"));
      now(iter_chains_on(whitelist))
   });

   async fn iter_chains_on(whitelist: impl Container<String>) -> ErrStr<()> {
      let chains = [AVALANCHE];
      for chain in chains {
         println!("\n=== Chain: {} ===", chain.blockchain());
         let tokens =
            match fetch_wallet_balances("pivot", chain).await {
               Ok(x) => x,
               Err(y) => panic!("Error: {y}")
         };
         let mut toks = tokens.result;
         println!("I received {} tokens", toks.len());
         toks.retain(|t| whitelist.contains(t));
         let tok = toks.first().unwrap();
         println!("{}\n{}\n\ntotal: {}",
                  tok.header(), enumerate_csv(&toks),
                  toks.iter().map(tvl).sum::<USD>());
      }
      Ok(())
   }
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

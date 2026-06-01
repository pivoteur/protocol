use reqwest::header::{ HeaderMap, HeaderValue };
use serde_json::from_str;

use book::{
   err_utils::{ErrStr,err_or},
   utils::{ get_env, now }
};
use crate::{
   paths::tsv_url,
   tables::{IxTable,index_table},
   types::{
      blockchains::Blockchain,
      tokens::moralis::Tokens,
      wallets::Wallet
   }
};
use super::utils::fetch_lines;

// ----- WALLETS ----------------------------------------------------

/// This fetches alles the wallets from github (manually updated for now)
pub async fn fetch_wallets_table(root_url: &str) -> ErrStr<IxTable> {
   let url = tsv_url(root_url, "wallets");
   let lines = fetch_lines(&url).await?;
   index_table(lines)
}

// Function to fetch native balance (e.g., ETH, MATIC)
pub fn fetch_wallets_balances(auth: &str) -> ErrStr<HashMap<Wallet, Tokens>> {

/*
This function models the following cURL command:

curl --request GET \
  --url 'https://deep-index.moralis.io/api/v2.2/wallets/{address}/tokens?chain=eth' \
  --header 'X-API-Key: test'
*/

   let aut = auth.to_uppercase();
   let addresses = get_env(&format!("{aut}_WALLET_ADDY"))?;
   let api_key = get_env(&format!("{aut}_MORALIS_API_KEY"))?;
   let wallets = from_str<Vec<Wallet>>(addresses)?;
   let c = reqwest::Client::new();
   let tokensz = filter_map_or(fetch_wallet_balances(c, api_key), wallets)?;
   Ok(wallets.into_iter().zip(tokensz.into_iter()).collect())
}

fn fetch_wallet_balances(api_key: &str)
      -> impl Fn(Client, Wallet) -> ErrStr<Tokens> {
   move | client: Client, w: Wallet | {
      let chain = w.blockchain.blockchain();
      let address = &w.address;
      let url0 = "https://deep-index.moralis.io/api/v2.2/wallets";
      let url = format!("{url0}/{address}/tokens?chain={chain}");
      let mut headers = HeaderMap::new();
      let api_hdr = err_or(HeaderValue::from_str(&api_key),
                           "Cannot insert MORALIS_API_KEY into header")?;
      headers.insert("X-API-Key", api_hdr);

      let res = 
         err_or(now(client.get(&url).headers(headers).send()),
                "Failed to send reqwest to moralis.io")?;
      let toks: Tokens =
         err_or(now(res.json()), "Cannot convert response from JSON")?;
      Ok(toks)
   }
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
      types::filters::{ Container, Sieve },
      utils::now
   };
   use crate::{
      fetchers::{
         test_helpers::test_functions::marshall,
         whitelist::fetch_whitelist
      },
      types::{ measurable::tvl, tokens::moralis::Blockchain::AVALANCHE }
   };

   create_testing!("fetchers::wallets");

   run!("fetch_wallets_table", " (fetch all wallets from github)", {
      let (root_url, _aliases) = marshall()?;
      let wallets = now(fetch_wallets_table(&root_url))?;
      println!("The wallets for {root_url} are:\n\n{}\n",
               wallets.as_csv());
   });

   run!("fetch_wallet_balances", " (no filter)", {
      now(iter_chains_on(Sieve))
   });

   run!("fetch_wallet_balances_whitelisted", {
      let whitelist =
         now(fetch_whitelist("pivot", "pivot-token-addresses.txt"))?;
      now(iter_chains_on(whitelist))
   });

   async fn iter_chains_on(whitelist: impl Container<String>) -> ErrStr<()> {
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
   async fn test_fetch_wallets_table_ok() -> ErrStr<()> {
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

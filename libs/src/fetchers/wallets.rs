use serde_json::{ json, Value };
use reqwest::{ Client, Response, header::HeaderValue };

use book::{
   csv_utils::{CsvHeader,CsvWriter,enumerate_csv},
   currency::usd::USD,
   err_utils::{ErrStr,err_or},
   string_utils::s,
   utils::get_env
};
use crate::{
   paths::tsv_url,
   tables::{IxTable,index_table},
   types::{
      measurable::tvl,
      quotes::Quotes,
      tokens::moralis::{
         TokenBalance,
         Blockchain,
         mk_native_coin,
         mk_rpc_request
      }
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

/// This fetches a wallet's balances from the blockchains via API
pub async fn fetch_wallet_balances(auth: &str, quotes: &Quotes,
                                   ch: Blockchain, debug: bool)
      -> ErrStr<Vec<TokenBalance>> {
   if debug {
      println!("RPC Fetch Wallet balances for {}", ch.blockchain());
   }
   let moralis_node_url = ch.url();
   let chain = ch.blockchain().to_uppercase();
   let aut = auth.to_uppercase();
   let addy = get_env(&format!("{aut}_WALLET_ADDY"))?;
   let api_key = get_env(&format!("{aut}_MORALIS_API_KEY"))?;
   let node = get_env(&format!("{aut}_MORALIS_{chain}_NODE"))?;
   let batch_payload = vec![
      mk_rpc_request(1, "eth_getBalance", json!([addy,"latest"])),
      mk_rpc_request(2, "eth_getTokenBalances", json!([{ "address": addy}] ))
   ];
   // Instantiate the HTTP client with Moralis Authentication headers
   let mut headers = reqwest::header::HeaderMap::new();

   let mut api_key_value =
      err_or(HeaderValue::from_str(&api_key),
             "Cannot make api_key HeaderValue")?;
   api_key_value.set_sensitive(true);

   headers.insert("X-API-Key", api_key_value);
   headers.insert("accept", HeaderValue::from_static("application/json"));

   let client =
      err_or(Client::builder().default_headers(headers).build(),
             "Cannot build client with api_key header")?;

   let response0: Response = err_or(client
        .post(format!("{moralis_node_url}/{node}"))
        .json(&batch_payload)
        .send()
        .await, "Could not fetch Moralis RPC response")?;
   if debug { println!("Received response {response0:?}"); }
   let response: Value = err_or(response0.json().await,
        "Unable to parse JSON from Moralis RPC response")?;
   if debug { println!("Parsed JSON to {response:?}"); }
   let mut ans = Vec::new();
   if let Some(responses) = response.as_array() {
      for res in responses {
         let id = res["id"].as_u64().unwrap_or(0);
            
         match id {
            1 => { 
               let tok = ch.protocol_token();
               let nat = parse_native_token(&tok, res, quotes, debug)?;
               ans.push(nat);
            },
            2 => ans.extend(parse_tokens(res, debug)?),
            x => panic!("Unknown Response ID received: {x}")
         }
      }
      Ok(ans)
   } else {
      Err(s("Unable to traverse JSON response of wallet balances"))
   }
}

fn parse_tokens(res: &Value, debug: bool) -> ErrStr<Vec<TokenBalance>> {
   // Parse Moralis Extended ERC-20 response mapping
   if let Ok(tokens) =
         serde_json::from_value::<Vec<TokenBalance>>(res["result"].clone()) {
      if debug {
         let first = tokens.first().unwrap();
         println!("ERC-20 Token Balances (Via Extended RPC):

{}
{}

total: {}",
first.header(), enumerate_csv(&tokens), tokens.iter().map(tvl).sum::<USD>())
      }
      Ok(tokens)
   } else {
      Err(s("Unable to parse tokens from JSON RPC response."))
   }
}

fn parse_native_token(prot: &str, res: &Value, quotes: &Quotes, debug: bool)
      -> ErrStr<TokenBalance> {
   // Parse native balance hex string (e.g. "0x...")
   if let Some(hex_bal) = res["result"].as_str() {
      let clean_hex = hex_bal.trim_start_matches("0x");
      if let Ok(wei_val) = u128::from_str_radix(clean_hex, 16) {
         let val = wei_val as f64 / 10_f64.powi(18);
         let qt = quotes.lookup(prot)?;
         let tok = mk_native_coin(prot, wei_val, qt);
         if debug {
            println!("
=======================================
Native Balance: {:.4}

{}
{}
=======================================", val, tok.header(), tok.as_csv());
         }
         Ok(tok)
      } else {
         Err(format!("Unable to parse hex {clean_hex}"))
      }
   } else {
      Err(format!("Unable to parse json {res:?}"))
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
      csv_utils::CsvWriter,
      date_utils::yesterday,
      utils::now,
      types::filters::{ Container, Sieve }
   };
   use crate::{
      fetchers::{
         quotes::fetch_quotes,
         test_helpers::test_functions::marshall
      },
      types::tokens::moralis::Blockchain::AVALANCHE
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

   async fn iter_chains_on(whitelist: impl Container<String>) -> ErrStr<()> {
      let chains = [AVALANCHE];
      let quotes = fetch_quotes(&yesterday()).await?;
      for chain in chains {
         println!("\n=== Chain: {} ===", chain.blockchain());
         let mut toks =
            fetch_wallet_balances("PIVOT", &quotes, chain, true).await?;
         toks.retain(|t| whitelist.contains(t));
         let tok = toks.first().unwrap();
         println!("{}\n{}", tok.header(), enumerate_csv(&toks));
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

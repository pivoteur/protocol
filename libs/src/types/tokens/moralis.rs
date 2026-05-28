use serde::{ Serialize, Deserialize, Deserializer };
use serde_json::Value;

use book::{
   currency::usd::{ USD,mk_usd },
   csv_utils::{CsvWriter,CsvHeader },
   string_utils::s,
   types::filters::Filter
};
use crate::types::measurable::{Measurable,tvl};

#[derive(Deserialize, Debug)]
pub struct Tokens { pub result: Vec<TokenBalance> }

#[derive(Deserialize, Debug)]
pub struct TokenBalance {
    symbol: String,
    balance: String,
    decimals: Option<u8>,
    token_address: String,

    #[serde(deserialize_with = "parse_float_to_usd")]
    usd_price: USD
}

pub fn mk_native_coin(sym: &str, bal: u128, quote: f32) -> TokenBalance {
   TokenBalance {
      symbol: s(sym),
      balance: format!("{bal}"),
      decimals: None,
      token_address: protocol_token_address(),
      usd_price: mk_usd(quote)
   }
}

// a type to send the RPC request for the wallet-information
// Generic envelope for JSON-RPC requests

#[derive(Debug,Serialize)]
pub struct RpcRequest {
    jsonrpc: &'static str,
    id: u32,
    method: &'static str,
    params: Value,
}

pub fn mk_rpc_request(id: u32, method: &'static str, params: Value)
      -> RpcRequest {
   let jsonrpc = "2.0";
   // let params = vec![parms];
   RpcRequest { jsonrpc, id, method, params }
}

pub enum Blockchain { AVALANCHE, BINANCE, ETHEREUM }
use Blockchain::*;

impl Blockchain {
   pub fn blockchain(&self) -> String {
      s(match self {
         AVALANCHE => "avalanche", BINANCE => "bsc", ETHEREUM => "eth" })
   }
   pub fn node(&self) -> String {
      format!("{} Mainnet", match self {
         AVALANCHE => "Avalanche",
         BINANCE   => "BNB Smart Chain",
         ETHEREUM  => "Ethereum" })
   }
   pub fn protocol_token(&self) -> String {
      s(match self { AVALANCHE => "AVAX", BINANCE => "BNB", ETHEREUM => "ETH" })
   }
   pub fn url(&self) -> String {
      format!("https://site1.moralis-nodes.com/{}", self.blockchain())
      // site2 is an alternative
   }
}

fn protocol_token_address() -> String {
   s("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee")
}

impl Filter<String> for TokenBalance {
   fn filter(&self) -> String { self.token_address.clone() }
}

impl Measurable for TokenBalance {
   fn sz(&self) -> f32 {
      let bal = self.bal();
      bal.parse().expect(&format!("{} is not a number", bal))
   }
   fn aug(&self) -> f32 { self.usd_price.amount }
}

impl CsvHeader for TokenBalance {
   fn header(&self) -> String { s("symbol,balance,quote,nav") }
}
impl CsvWriter for TokenBalance {
   fn ncols(&self) -> usize { 4 }
   fn as_csv(&self) -> String {
      format!("{},{},{},{}",self.symbol,self.bal(),self.usd_price,tvl(self))
   }
}

impl TokenBalance {
   // Formats a raw string balance using the provided decimals
   pub fn bal(&self) -> String {
      let raw_balance = &self.balance;
      let deci = &self.decimals;
      match raw_balance.parse::<f64>() {
         Ok(val) => {
            let dec = deci.unwrap_or(18);
            let formatted = val / 10.0_f64.powi(dec as i32);
            format!("{:.4}", formatted)
            // Truncate to 4 decimal places for readability
         }
         Err(_) => s(raw_balance)
      }
   }
}

fn parse_float_to_usd<'de, D>(deserializer: D) -> Result<USD, D::Error>
      where D: Deserializer<'de> {
    // First, deserialize the JSON field into a standard Rust String
    let s = String::deserialize(deserializer)?;
    
    // Second, parse the string using your own logic
    let parsed_float = s.parse::<f32>().map_err(serde::de::Error::custom)?;
    
    // Finally, return your constructed type
    Ok(mk_usd(parsed_float))
}


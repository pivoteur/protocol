use std::fmt;

use strum::EnumString;

use book::string_utils::s;

#[derive(Clone,Debug,PartialEq,Eq,Hash,EnumString)]
#[strum(serialize_all = "UPPERCASE", ascii_case_insensitive)]
pub enum Blockchain { AURORA, AVALANCHE, BINANCE, CARDANO, ETHEREUM, NEAR }
use Blockchain::*;

impl Blockchain {
   pub fn blockchain(&self) -> String {
      s(match self {
         AURORA    => "aurora",
         AVALANCHE => "avalanche",
         BINANCE   => "bsc",
         CARDANO   => "cardano",
         ETHEREUM  => "eth",
         NEAR      => "near"
      })
   }
   pub fn node(&self) -> String {
      format!("{} Mainnet", match self {
         AURORA    => "aurora",
         AVALANCHE => "Avalanche",
         BINANCE   => "BNB Smart Chain",
         CARDANO   => "Cardano",
         ETHEREUM  => "Ethereum",
         NEAR      => "Near"
      })
   }
   pub fn protocol_token(&self) -> String {
      s(match self {
         AURORA    => "AURORA",
         AVALANCHE => "AVAX",
         BINANCE   => "BNB",
         CARDANO   => "ADA",
         ETHEREUM  => "ETH",
         NEAR      => "NEAR"
      })
   }
   pub fn url(&self) -> String {
      s(match self {
         AVALANCHE => "https://api.avax.network/ext/bc/C/rpc",
         BINANCE   => "https://bsc-dataseed.bnbchain.org",
         x         => panic!("Blockchain {x} not supported")
      })
   }
   pub fn chain_id(&self) -> u64 { 
      match self {
         AVALANCHE => 43114,
         BINANCE   => 56,
         x         => panic!("No chain_id for {x}")
      }
   }
}

impl fmt::Display for Blockchain {
   fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      write!(formatter, "{:?}", self)
   }
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use book::err_utils::{ ErrStr, err_or };

   #[test] fn test_parse_ok() -> ErrStr<()> {
      let ans: Blockchain =
         err_or("avalanche".parse(),
                "Cannot parse 'avalanche' blockchain")?;
      assert_eq!(AVALANCHE, ans);
      Ok(())
   }

   #[test] fn fail_parse() {
      let ans: ErrStr<Blockchain> =
         err_or("blerg".parse(), "Cannot parse 'blerg' blockchain");
      assert!(ans.is_err());
   }
}

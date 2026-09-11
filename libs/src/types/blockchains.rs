use std::{ fmt, str::FromStr };

use book::{ err_utils::ErrStr, string_utils::s };

#[derive(Clone, Debug,PartialEq)]
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
      s("https://api.avax.network/ext/bc/C/rpc") // works for everybody?
   }
   pub fn chain_id(&self) -> u64 { 
      match self {
         AVALANCHE => 43114,
         x => panic!("No chain_id for {x}")
      }
   }
}

impl fmt::Display for Blockchain {
   fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      write!(formatter, "{:?}", self)
   }
}

impl FromStr for Blockchain {
   type Err = String;
   fn from_str(elt: &str) -> ErrStr<Self> {
      match elt.to_uppercase().as_str() {
         "AURORA"    => Ok(AURORA),
         "AVALANCHE" => Ok(AVALANCHE),
         "BINANCE"   => Ok(BINANCE),
         "CARDANO"   => Ok(CARDANO),
         "ETHEREUM"  => Ok(ETHEREUM),
         "NEAR"      => Ok(NEAR),
         _           => Err(format!("Unable to parse blockchain from {elt}"))
      }
   }
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;

   #[test] fn test_parse_ok() -> ErrStr<()> {
      let ans: Blockchain = "avalanche".parse()?;
      assert_eq!(AVALANCHE, ans);
      Ok(())
   }

   #[test] fn fail_parse() {
      let ans: ErrStr<Blockchain> = "blerg".parse();
      assert!(ans.is_err());
   }
}

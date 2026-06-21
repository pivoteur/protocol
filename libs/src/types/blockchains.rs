use std::{ fmt, str::FromStr };

use book::{ err_utils::ErrStr, string_utils::s };

#[derive(Clone, Debug,PartialEq)]
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

impl fmt::Display for Blockchain {
   fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      write!(formatter, "{:?}", self)
   }
}

impl FromStr for Blockchain {
   type Err = String;
   fn from_str(elt: &str) -> ErrStr<Self> {
      match elt.to_uppercase().as_str() {
         "AVALANCHE" => Ok(AVALANCHE),
         "BINANCE"   => Ok(BINANCE),
         "ETHEREUM"  => Ok(ETHEREUM),
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

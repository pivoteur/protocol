use std::fmt;

use serde::Deserialize;

use book::{ err_utils::{err_or,ErrStr}, string_utils::s };

#[derive(Clone, Deserialize, Debug, PartialEq, Hash)]
pub enum Blockchain { AVALANCHE, BINANCE, ETHEREUM }

impl Eq for Blockchain { }

impl fmt::Display for Blockchain {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f, "{}", self.blockchain())
   }
}

use Blockchain::*;

pub fn parse_blockchain(b: &str) -> ErrStr<Blockchain> {
   let big_b = b.to_uppercase();
   let ans: Blockchain =
      err_or(big_b.parse(), &format!("No such blockchain: {b}"))?;
   Ok(ans)
}

impl Blockchain {
   pub fn blockchain(&self) -> String {
      s(match self {
         AVALANCHE => "avalanche", BINANCE => "bsc", ETHEREUM => "eth" })
   }
   pub fn protocol_token(&self) -> String {
      s(match self { AVALANCHE => "AVAX", BINANCE => "BNB", ETHEREUM => "ETH" })
   }
}


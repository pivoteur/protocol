use serde::Deserialize;
use serde_json::from_str;

use book::{
   csv_utils::{ CsvHeader, CsvWriter },
   err_utils::{ ErrStr, err_or },
   string_utils::s
};
use super::{ blockchains::Blockchain, util::Address };

#[derive(Clone, Deserialize, Debug, PartialEq, Hash)]
pub struct Wallet {
   wallet: String,
   blockchain: Blockchain,
   address: Address
}

impl Eq for Wallet { }

impl CsvHeader for Wallet {
   fn header(&self) -> String { s("wallet,blockchain,address") }
}

impl CsvWriter for Wallet {
   fn ncols(&self) -> usize { 3 }
   fn as_csv(&self) -> String {
      format!("{},{},{}",
              self.wallet, self.blockchain.blockchain(), self.address)
   }
}

impl Wallet {
   // inject wallet-information into the URL
   pub fn build_url<'a>(&self,
                        f: impl Fn(&'a Blockchain, &'a Address) -> String)
         -> String {
      f(&self.blockchain, &self.address)
   }
}

pub fn parse_wallets(json: &str) -> ErrStr<Vec<Wallet>> {
   err_or(from_str::<Vec<Wallet>>(json),
          "Could not parse JSON to wallets")
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod sample_data {
   use super::*;

   pub fn sample_wallets_str() -> String { r#"[
   { "wallet": "treasury", "blockchain": "AVALANCHE", "address": "0x123" },
   { "wallet": "binance", "blockchain": "BINANCE", "address": "0x123" },
   { "wallet": "uniswap", "blockchain": "ETHEREUM", "address": "0x123" }
]"#
   }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use super::sample_data::sample_wallets_str;

   use paste::paste;
   use book::create_testing;

   create_testing!("types::wallets");

   run!("parse_wallets", " (JSON)", {
      let json = sample_wallets_str();
      let walleto = parse_wallets(&json)?;
      println!("The wallets parsed from 

{}

are

{}", json, enumerate_csv(&walleto));
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use super::sample_data::sample_wallets_str;
   use book::create_testing;
   use crate::types::Blockchain::ETEREUM;

   #[test] fn test_parse_wallet() -> ErrStr<()> {
      let walleto = parse_wallets(&sample_wallets_str())?;
      assert_eq!(3, walleto.len());
      assert_eq!(ETHEREUM, walleto[2].blockchain);
      assert_eq!("binance", &walleto[1].wallet);
      assert_eq!("0x123", &wallet0[0].address);
      Ok(())
   }
}



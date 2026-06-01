use serde::Deserialize;

use super::{ blockchains::{Blockchain,Blockchain::*}, util::Address };

#[derive(Deserialize, Debug, Eq, Hash)]
pub struct Wallet {
   pub name: String,
   pub blockchain: Blockchain,
   pub address: Address
}


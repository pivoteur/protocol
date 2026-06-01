use serde::Deserialize;

use book::{
   currency::usd::{ USD,mk_usd },
   csv_utils::{CsvWriter,CsvHeader },
   err_utils::ErrStr,
   num_utils::parse_num,
   string_utils::s,
   types::filters::Filter
};
use crate::types::{ measurable::{Measurable,tvl}, util::Token };

#[derive(Deserialize, Debug, Eq, Hash)]
pub enum Blockchain { AVALANCHE, BINANCE, ETHEREUM }
use Blockchain::*;

impl Blockchain {
   pub fn blockchain(&self) -> String {
      s(match self {
         AVALANCHE => "avalanche", BINANCE => "bsc", ETHEREUM => "eth" })
   }
   pub fn protocol_token(&self) -> String {
      s(match self { AVALANCHE => "AVAX", BINANCE => "BNB", ETHEREUM => "ETH" })
   }
}


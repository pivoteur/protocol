use serde::Serialize;
use serde_with::{ serde_as, DisplayFromStr };
use book::currency::usd::USD;

#[serde_as]
#[derive(Debug, Clone, Serialize)]
pub struct TVL {
   #[serde_as(as = "DisplayFromStr")]
   total: USD,
   #[serde_as(as = "DisplayFromStr")]
   #[serde(rename = "virtual")]
   virtual_amt: USD,
   #[serde_as(as = "DisplayFromStr")]
   available: USD,
   #[serde_as(as = "DisplayFromStr")]
   reserve: USD // how much of the protocol token hasn't been converted
}

pub fn mk_tvl(total: USD, virt: USD, available: USD, reserve: USD) -> TVL {
   TVL { total, virtual_amt: virt, available, reserve }
}

// ----- TESTS -------------------------------------------------------

#[cfg(not(tarpaulin_include))]
pub mod sample_data {
   use super::*;
   use book::currency::usd::{ mk_usd, no_monay };

   pub fn sample_btc_eth_tvl() -> TVL {
      mk_tvl(mk_usd(117034.93), mk_usd(27323.13), mk_usd(6.9), no_monay())
   }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use paste::paste;
   use book::{ create_testing, err_utils::{ err_or, ErrStr } };
   use super::sample_data::sample_btc_eth_tvl;

   create_testing!("types::pools::tvl");

   run!("mk_tvl", {
      let tvl = sample_btc_eth_tvl();
      let ans = err_or(serde_json::to_string_pretty(&tvl),
                       &format!("Cannot JSONify {tvl:?}"))?;
      println!("JSON of tvl is:\n{ans}");
   });
}

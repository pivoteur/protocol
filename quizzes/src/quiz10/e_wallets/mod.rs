use std::collections::HashMap;

use book::{ err_utils::ErrStr, string_utils::s, utils::get_args };

use libs::processors::process_wallet_balances;

fn version() -> String { s("1.00") }
fn app_name() -> String { s("wallets") }
fn usage() -> ErrStr<String> {
   let app = app_name();
   println!("{}, version: {}

Usage:

$ {} [--debug] <protocol>

where:

* [--debug] (optional) shows debugging information
* <protocol> the protocol on which the wallet-query occurs

Environment:

In order for {} to run, the following environmental variables must be set:

* <protocol>_WALLET_ADDY: the address of the wallet to query token-balances
* <protocol>_MORALIS_API_KEY: the Moralis API KEY to query the blockchain
", app, version(), app, app);
   Err("Need <protocol> argument and environment set")
}

pub fn runoff_with_args() -> ErrStr<()> {
   // TODO: implementation is where now?
}

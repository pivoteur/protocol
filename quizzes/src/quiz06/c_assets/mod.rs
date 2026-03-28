use chrono::NaiveDate;

use std::collections::HashMap;

use book::{
   currency::usd::{USD,mk_usd},
   date_utils::parse_date,
   err_utils::ErrStr,
   num_utils::parse_num,
   table_utils::col,
   utils::get_env
};

use libs::{
   fetchers::{fetch_quotes,fetch_asset_table_tvls,fetch_wallets},
   tables::IxTable,
   types::{ quotes::Quotes, util::{Token, TVLs} }
};

fn version() -> String { "1.07".to_string() }
fn app_name() -> String { "assets".to_string() }

fn usage() -> ErrStr<()> { println!("
{}, version: {}

Prints the assets of the protocol, giving the TVL

Usage:

	$ {} <auth> <date>

where

* <auth> is protocol authenticator
* <date> the date of the assets-value, e.g.: $LE_DATE
", app_name(), version(), app_name());
   Err("Need <auth> <date> arguments!".to_string())
}

async fn compute_assets(auth: &str, dt: &NaiveDate) -> ErrStr<Vec<USD>> {
   let root_url = get_env(&format!("{}_URL", auth.to_uppercase()))?;
   let qts = fetch_quotes(&dt).await?;
   let wlts = fetch_wallets(&root_url).await?;
   let tvls = fetch_asset_table_tvls(&root_url).await?;
   let amts = amounts(&wlts, &tvls);
   Ok(row(&qts, &amts, &tvls))
}

fn amounts(wallets: &IxTable, assets: &TVLs) -> HashMap<Token,f32> {
   assets.iter().filter_map(amt(wallets)).collect()
}

fn row(quotes: &Quotes, amts: &HashMap<Token,f32>, tvls: &TVLs) -> Vec<USD> {
   tvls.iter().filter_map(tvl(amts, quotes)).collect()
}

fn tvl(amts: &HashMap<Token,f32>, quotes: &Quotes)
      -> impl Fn(&(Token, USD)) -> Option<USD> {
   move |(t, default)| {
// tvl is an interesting problem, type-wise, come to find.

// first, the quote ... do we want to fail if we can't lookup the quote?
// ... no, because we don't care what the price of, e.g. QI, is anymore
// we only care about the prices on the assets we currently track

      let ans = quotes.lookup(&t).and_then(|q| {

// now we need an amount from the wallets, if not ...
// then return 0.0, as we have no e.g. BNB in our wallets anymore

         let amt = amts.get(t).unwrap_or(&0.0);
         Ok(mk_usd(q * amt))
      }).unwrap_or(default.clone());

      Some(ans)  // so this says we always arrive at an answer
   }
}

fn amt(wallets: &IxTable) -> impl Fn(&(Token, USD)) -> Option<(Token, f32)> {
   move | (t, _tvl) | {
      col(wallets, &t)
         .and_then(|c|
             Some((t.to_string(),
                   c.into_iter().filter_map(|n| parse_num(&n).ok()).sum())))
   }
}

fn output_line(dt: &NaiveDate, row: &Vec<USD>) -> String {
   let dollaz: Vec<String> = row.into_iter().map(|d| format!("{d}")).collect();
   format!("{dt}\t{}", dollaz.join("\t"))
}

// ----- TESTS -------------------------------------------------------

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {

   use super::*;

   use book::{ date_utils::yesterday, utils::get_args };

   async fn run_compute_assets(auth: &str, dt: &str) -> ErrStr<usize> {
      let date = parse_date(dt)?;
      let tvls = compute_assets(auth, &date).await?;
      let row = output_line(&date, &tvls);
      println!("{row}\n");
      Ok(1)
   }

   pub async fn runoff_get_args() -> ErrStr<()> {
      let args = get_args();
      if let [auth, dt] = args.as_slice() {
         let _ = run_compute_assets(&auth, &dt).await?;
         Ok(())
      } else {
         usage()
      }
   }

   pub async fn runoff() -> ErrStr<usize> {
      println!("\nquizzes::quiz06::c_assets functional test\n");
      let yday = yesterday();
      println!("\tasset row is:\n");
      run_compute_assets("pivot", &format!("{yday}")).await
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   use book::{
      date_utils::yesterday,
      string_utils::{to_string,words}
   };
   use libs::{ tables::index_table, types::quotes::mk_quotes };

   #[tokio::test]
   async fn test_compute_assets_ok() {
      let yday = yesterday();
      let ans = compute_assets("pivot", &yday).await;
      assert!(ans.is_ok());
   }

   #[tokio::test]
   async fn fail_compute_assets() {
      let yday = yesterday();
      let ans = compute_assets("adsff", &yday).await;
      assert!(ans.is_err());
   }

   fn wallets() -> ErrStr<IxTable> {
      let minimi = 
"pool	BTC	ETH	UNDEAD
BTC+ETH	1.0	34.0	0.0
BTC+UNDEAD	0.1	0.0	6250000
ETH+UNDEAD	0.0	5	6250000";
      let lines: Vec<String> =
         minimi.split("\n").into_iter().map(to_string).collect();
      index_table(lines)
   }

   fn hdrs() -> Vec<String> { words("BTC ETH UNDEAD") }
   fn assets() -> TVLs {
      let amts: Vec<USD> =
         vec![110000.0, 20075.0, 78823.29].into_iter().map(mk_usd).collect();
      hdrs().into_iter().zip(amts.into_iter()).collect()
   }

   fn prices() -> Vec<f32> { vec![67123.74, 0.001542, 2030.83] }
   fn quotes() -> Quotes {
      let qts: HashMap<Token, f32> =
         hdrs().into_iter().zip(prices().into_iter()).collect();
      let yday = yesterday();
      mk_quotes(yday, qts)
   }

   fn amts() -> Vec<f32> { vec![1.1, 39.0, 12500000.0] }

   #[test]
   fn test_amounts() -> ErrStr<()> {
      let w = wallets()?;
      let ans = amounts(&w, &assets());
      let hdr_amts: Vec<(String, f32)> =
         hdrs().into_iter().zip(amts().into_iter()).collect();
      for (hdr,amt) in hdr_amts {
         let tokin = ans.get(&hdr);
         assert!(tokin.is_some());
         assert_eq!(&amt, tokin.unwrap(), "For {hdr}, amounts mismatch");
      }
      Ok(())
   }

   #[test]
   fn test_row() -> ErrStr<()> {
      let w = wallets()?;
      let ans = amounts(&w, &assets());
      let r = row(&quotes(), &ans, &assets());
      assert_eq!(3, r.len());
      for (ix, amt) in amts().into_iter().enumerate() {
         assert_eq!(prices()[ix] * amt, r[ix].amount,
                    "For {}, TVLs mismatch", hdrs()[ix]);
      }
      Ok(())
   }
}


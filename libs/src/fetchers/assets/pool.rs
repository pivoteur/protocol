use std::collections::HashMap;

use chrono::NaiveDate;

use book::{
   csv_utils::CsvWriter,
   currency::usd::USD,
   date_utils::parse_date,
   err_utils::{err_or,ErrStr},
   num_utils::parse_commaless,
   parse_utils::parse_str,
   table_utils::{cols,row,rows,ingest}
};

use crate::{
   collections::assets::Assets,
   fetchers::{ pivots::fetch_pivots, utils::fetch_lines },
   paths::pool_assets_url,
   types::{
      aliases::Aliases,
      tokens::coins::{Coin,mk_coin},
      comps::{Composition,mk_composition},
      pivots::{Pivot,pivot_assets},
      pools::Pool,
      quotes::Quotes,
      util::{Token,Blockchain}
   }
};

// ----- POOL ASSETS ------------------------------------------------

// this one fetches all the assets of the pool

pub async fn fetch_assets(root_url: &str, pool: &Pool, aliases: &Aliases,
                          debug: bool) -> ErrStr<Composition> {
   let url = pool_assets_url(root_url, pool);
   let lines = fetch_lines(&url).await?;
   let table = ingest(parse_date, parse_str, parse_str, &lines, "\t")?;
   if debug {
      println!("fetchers::assets::pool::fetch_assets:
	for {pool}, assets table size: {}", table.data.len());
   }
   let hdrs = aliases.enum_headers(cols(&table));
   let max_date = rows(&table).iter().max().cloned()
                              .ok_or(format!("No max_date for {pool}"))?;
   let top = row(&table, &max_date)
                .ok_or(format!("No row for date {max_date}"))?;
   let blk = top[hdrs["blockchain"]].clone();
   let (p, s) = pool.as_tuple();
   let primary = buidl_asset("primary", &top[hdrs[&p]], qt_f(&top, &hdrs),
                             &blk, &p, &max_date, debug)?;
   let h_s = hdrs.get(&s)
      .expect(&format!("No header labeled {}; headers are {:?}", s, hdrs));
   let s_amt = &top.get(*h_s).expect(&format!("No value at index {}", h_s));
   let f = qt_f(&top, &hdrs);
   let pivot = buidl_asset("pivot", s_amt, f, &blk, &s, &max_date, debug)?;
   Ok(mk_composition(&primary, &pivot))
}

fn qt_f<'a>(v: &'a Vec<String>, hdrs: &'a HashMap<String, usize>)
      -> impl Fn(&'a Token) -> ErrStr<USD> {
   |t: &'a Token| {
      let q = &v[hdrs[&format!("{t} quote")]];
      let quote: USD = err_or(q.parse(), &format!("No quote for {t}"))?;
      Ok(quote)
   }
}
   
fn buidl_asset<'a>(asset_type: &str, amount: &str,
                   q: impl Fn(&'a Token) -> ErrStr<USD>, 
                   blk: &Blockchain, t: &'a Token, dt: &NaiveDate, debug: bool)
      -> ErrStr<Coin> {
   let amt = parse_commaless(amount)?;
   let quote = q(t)?;
   let asset = mk_coin(&(blk.clone(), t.clone()), amt, &quote, dt);
   if debug { println!("\t{asset_type} asset is {}", asset.as_csv()); }
   Ok(asset)
}

// ----- AVAILABLE ASSETS -------------------------------------------------

// this gets the assets and the open pivots (so we compute available assets)

pub async fn fetch_assets_and_open_pivots
      (root_url: &str, quotes: &Quotes, pool: &Pool, debug: bool)
      -> ErrStr<(Composition, Vec<Pivot>)> {
   let aliases = &quotes.aliases;
   let pool_assets = fetch_assets(&root_url, pool, aliases, debug).await?;
   let ((opens, _closes), _max_date) =
      fetch_pivots(&root_url, pool, aliases, debug).await?;
   Ok((pool_assets, opens))
}

pub async fn available_assets_fetcher
      (subtractor: impl Fn(&mut Assets, &Coin), root_url: &str,
       quotes: &Quotes, pool: &Pool, debug: bool) -> ErrStr<Composition> {
   let (pool_assets, opens) =
      fetch_assets_and_open_pivots(root_url, &quotes, pool, debug).await?;
   let mut available = pool_assets.as_assets();
   let all_opens = pivot_assets(&opens)?;
   for a in all_opens.assets() {
      subtractor(&mut available, &a);
   }
   available.as_composition(pool, quotes)
}

pub fn subtractor(assets: &mut Assets, coin: &Coin) { assets.subtract(coin); }

pub async fn fetch_available_assets(root_url: &str, q: &Quotes, p: &Pool,
                                    debug: bool) -> ErrStr<Composition> {
   available_assets_fetcher(subtractor, root_url, q, p, debug).await
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{
      create_testing,
      csv_utils::CsvWriter,
      date_utils::yesterday,
      utils::now
   };
   use crate::{
      fetchers::{quotes::fetch_quotes,test_helpers::test_functions::marshall},
      types::pools::pool_from_str
   };

   create_testing!("fetchers::assets::pool");

   run!("fetch_pool_assets", {
      let (root_url, a) = marshall()?;
      let btc_eth = pool_from_str("btc-eth")?;
      let pa = now(fetch_assets(&root_url, &btc_eth, &a, true))?;
      println!("BTC+ETH pivot pool assets are:\n{}", pa.as_csv());
   });

   run!("fetch_available_assets", {
      let yday = yesterday();
      let (url, _) = marshall()?;
      let quotes = now(fetch_quotes(&yday))?;
      let pool = pool_from_str("btc-eth")?;
      let comp = now(fetch_available_assets(&url, &quotes, &pool, true))?;
      println!("\nAvailable BTC+ETH assets\n{}", comp.as_csv());
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use book::{
      currency::usd::no_monay,
      csv_utils::CsvWriter,
      date_utils::yesterday,
      utils::{ composer, deref }
   };
   use crate::{
      fetchers::{quotes::fetch_quotes,test_helpers::test_functions::marshall},
      types::{ measurable::tvl, pools::pool_from_str }
   };

   // ----- ALL POOL ASSETS ------------------------------------------

   #[tokio::test]
   async fn test_fetch_assets_ok() -> ErrStr<()> {
      let (root_url, a) = marshall()?;
      let btc_eth = pool_from_str("btc-eth")?;
      let mb_assets = fetch_assets(&root_url, &btc_eth, &a, true).await;
      assert!(mb_assets.is_ok());
      Ok(())
   }

   #[tokio::test]
   async fn test_fetch_assets() -> ErrStr<()> {
      let (root_url, a) = marshall()?;
      let btc_eth = pool_from_str("btc-eth")?;
      let assets = fetch_assets(&root_url, &btc_eth, &a, true).await?;
      assert!(assets.tvl().amount() > 0.0);
      assert_eq!("BTC+ETH", assets.pool_name());
      Ok(())
   }

   // ----- AVAILABLE ASSETS ----------------------------------------

   async fn fetchme() -> ErrStr<Composition> {
      let yday = yesterday();
      let (url, _) = marshall()?;
      let quotes = fetch_quotes(&yday).await?;
      let pool = pool_from_str("btc-eth")?;
      fetch_available_assets(&url, &quotes, &pool, true).await
   }

   #[tokio::test]
   async fn test_fetch_available_assets_ok() {
      let fetchèð = fetchme().await;
      assert!(fetchèð.is_ok());
   }  

   fn tvlr() -> impl Fn(Pivot) -> USD {
      composer(deref(tvl), composer(Result::unwrap, deref(Pivot::committed)))
   }  

   async fn enfetchme() -> ErrStr<(Composition, Vec<Pivot>)> {
      let yday = yesterday();
      let (url, _) = marshall()?;
      let pool = pool_from_str("btc-eth")?;
      let quotes = fetch_quotes(&yday).await?;
      fetch_assets_and_open_pivots(&url, &quotes, &pool, true).await
   }

   #[tokio::test]
   async fn test_fetch_available_assets() -> ErrStr<()> {
      let (comp, _pivs) = enfetchme().await?;
      assert!(comp.tvl() > no_monay(), "BTC+ETH Pool composition zero!");
      Ok(())
   }  

   #[tokio::test]
   async fn test_enfetchify_has_pivots() -> ErrStr<()> {
      let (_comp, pivs) = enfetchme().await?;
      assert!(!pivs.is_empty(), "No open BTC+ETH pivots?");
      Ok(())
   }

   #[tokio::test]
   async fn test_enfetchify_pivots() -> ErrStr<()> {
      let (_comp, pivs) = enfetchme().await?;
      let pivs_tvls: USD = pivs.into_iter().map(tvlr()).sum();
      assert!(pivs_tvls > no_monay(), "BTC+ETH pivots amount to $0.00?");
      Ok(())
   }

   #[tokio::test]
   async fn test_enfetchify_assets_and_pivots() -> ErrStr<()> {
      let comp = fetchme().await?;
      assert!(comp.tvl() > no_monay(),
              "BTC+ETH pivot pool:
{}
has negative balance: {}", comp.as_csv(), comp.tvl());
      Ok(())
   }
}

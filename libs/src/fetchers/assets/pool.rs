use std::collections::HashMap;

use chrono::NaiveDate;

use book::{
   currency::usd::USD,
   date_utils::parse_date,
   err_utils::{err_or,ErrStr},
   num_utils::parse_commaless,
   parse_utils::parse_str,
   table_utils::{cols,row,rows,ingest},
   utils::get_env
};

use crate::{
   fetchers::{
      pivots::fetch_pivots,
      utils::{ enlowerify, enupperify, fetch_lines }
   },
   paths::pool_assets_url,
   types::{
      aliases::Aliases,
      blockchains::{Blockchain,mk_blockchain},
      comps::{Composition,mk_composition,from_assets},
      pivots::{Pivot,pivot_assets},
      quotes::Quotes,
      tokens::coins::{Coin,mk_coin},
      util::{Token,Pool}
   }
};

// ----- POOL ASSETS ------------------------------------------------

// this one fetches all the assets of the pool

pub async fn fetch_assets(root_url: &str, primary: &str, pivot: &str,
                          aliases: &Aliases) -> ErrStr<Composition> {
   let (pri, seggs) = enlowerify(primary, pivot);
   let url = pool_assets_url(root_url, &pri, &seggs);
   let lines = fetch_lines(&url).await?;
   let table = ingest(parse_date, parse_str, parse_str, &lines, "\t")?;
   let hdrs = aliases.enum_headers(cols(&table));
   let (p, s) = enupperify(primary, pivot);
   let max_date = rows(&table).iter().max().cloned()
                              .ok_or(format!("No max_date for {p}+{s}"))?;
   let top = row(&table, &max_date)
                .ok_or(format!("No row for date {max_date}"))?;
   let blk = parse_blockchain(&top[hdrs["blockchain"]])?;
   let primary = buidl_asset(&top[hdrs[&p]], qt_f(&top, &hdrs),
                             &blk, &p, &max_date)?;
   let h_s = hdrs.get(&s)
      .expect(&format!("No header labeled {}; headers are {:?}", s, hdrs));
   let s_amt = &top.get(*h_s).expect(&format!("No value at index {}", h_s));
   let f = qt_f(&top, &hdrs);
   let pivot = buidl_asset(s_amt, f, &blk, &s, &max_date)?;
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
   
fn buidl_asset<'a>(amount: &str, q: impl Fn(&'a Token) -> ErrStr<USD>, 
                   blk: &Blockchain, t: &'a Token, dt: &NaiveDate)
      -> ErrStr<Coin> {
   let amt = parse_commaless(amount)?;
   let quote = q(t)?;
   Ok(mk_coin(&(blk.clone(), t.clone()), amt, &quote, dt))
}

// ----- AVAILABLE ASSETS -------------------------------------------------

// this gets the assets and the open pivots (so we compute available assets)

async fn enfetchify(auth: &str, quotes: &Quotes, pool: &Pool)
      -> ErrStr<(Composition, Vec<Pivot>)> {
   let aut = auth.to_uppercase();
   let root_url = get_env(&format!("{aut}_URL"))?;
   let aliases = &quotes.aliases;
   let (prim, piv) = pool;
   let pool_assets = fetch_assets(&root_url, &prim, &piv, aliases).await?;
   let ((opens, _closes), _max_date) =
      fetch_pivots(&root_url, &prim, &piv, aliases).await?;
   Ok((pool_assets, opens))
}

pub async fn fetch_available_assets(auth: &str, quotes: &Quotes, pool: &Pool)
      -> ErrStr<Composition> {
   let (pool_assets, opens) = enfetchify(auth, &quotes, pool).await?;
   let mut available = pool_assets.as_assets();
   let all_opens = pivot_assets(&opens)?;
   for a in all_opens.assets() {
      available.subtract(&a);
   }
   available.update_prices(&quotes)?;
   from_assets(&available.assets())
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
      types::util::pool_from_str
   };

   create_testing!("fetchers::assets::pool");

   run!("fetch_pool_assets", {
      let (root_url, a) = marshall()?;
      let pa = now(fetch_assets(&root_url, "btc", "eth", &a))?;
      println!("BTC+ETH pivot pool assets are:\n{}", pa.as_csv());
   });

   run!("fetch_available_assets", {
      let yday = yesterday();
      let quotes = now(fetch_quotes(&yday))?;
      let pool = pool_from_str("btc-eth")?;
      let comp = now(fetch_available_assets("pivot", &quotes, &pool))?;
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
      types::{
         measurable::tvl,
         util::pool_from_str
      }
   };

   // ----- ALL POOL ASSETS ------------------------------------------

   #[tokio::test]
   async fn test_fetch_assets_ok() -> ErrStr<()> {
      let (root_url, a) = marshall()?;
      let mb_assets = fetch_assets(&root_url, "btc", "eth", &a).await;
      assert!(mb_assets.is_ok());
      Ok(())
   }

   #[tokio::test]
   async fn test_fetch_assets() -> ErrStr<()> {
      let (root_url, a) = marshall()?;
      let assets = fetch_assets(&root_url, "btc", "eth", &a).await?;
      assert!(assets.tvl().amount() > 0.0);
      assert_eq!("BTC+ETH", assets.pool_name());
      Ok(())
   }

   // ----- AVAILABLE ASSETS ----------------------------------------

   async fn fetchme() -> ErrStr<Composition> {
      let yday = yesterday();
      let quotes = fetch_quotes(&yday).await?;
      let pool = pool_from_str("btc-eth")?;
      fetch_available_assets("pivot", &quotes, &pool).await
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
      let pool = pool_from_str("btc-eth")?;
      let quotes = fetch_quotes(&yday).await?;
      enfetchify("pivot", &quotes, &pool).await
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

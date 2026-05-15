use std::collections::HashMap;

use chrono::NaiveDate;

use book::{
   currency::usd::USD,
   date_utils::parse_date,
   err_utils::{err_or,ErrStr},
   num_utils::parse_commaless,
   parse_utils::parse_str,
   table_utils::{cols,row,rows,ingest}
};

use crate::{
   fetchers::utils::{ enlowerify, enupperify, fetch_lines },
   paths::pool_assets_url,
   types::{
      aliases::Aliases,
      coins::{Coin,mk_coin},
      comps::{Composition,mk_composition},
      util::{Token,Blockchain}
   }
};

// ----- POOL ASSETS ------------------------------------------------

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
   let blk = top[hdrs["blockchain"]].clone();
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

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{
      create_testing,
      csv_utils::CsvWriter,
      utils::now
   };
   use crate::fetchers::test_helpers::test_functions::marshall;

   create_testing!("fetchers::assets::pool");

   run!("fetch_pool_assets", {
      let (root_url, a) = marshall()?;
      let pa = now(fetch_assets(&root_url, "btc", "eth", &a))?;
      println!("BTC+ETH pivot pool assets are:\n{}", pa.as_csv());
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use crate::fetchers::test_helpers::test_functions::marshall;

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
      assert!(assets.tvl().amount > 0.0);
      assert_eq!("BTC+ETH", assets.pool_name());
      Ok(())
   }
}

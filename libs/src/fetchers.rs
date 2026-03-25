/// fetch data from REST endpoints

use std::collections::HashMap;

use chrono::NaiveDate;

use book::{
   currency::usd::USD,
   date_utils::{parse_date,datef},
   err_utils::{err_or,ErrStr},
   list_utils::tail,
   num_utils::{parse_num,parse_commaless},
   parse_utils::parse_str,
   rest_utils::read_rest,
   table_utils::{Table,cols,row,rows,ingest},
   utils::pred
};

use super::{
   paths::{open_pivot_path,quotes_url,pool_assets_url,tsv_url},
   tables::{IxTable,index_table},
   types::{
      aliases::{Aliases,aliases},
      pivots::{Pivot,parse_pivot},
      quotes::{Quotes,mk_quotes},
      coins::{Coin,mk_coin},
      comps::{Composition,mk_composition},
      util::{Token,Blockchain}
   }
};

pub async fn fetch_wallets(root_url: &str) -> ErrStr<IxTable> {
   let url = tsv_url(root_url, "wallets");
   let lines = fetch_lines(&url).await?;
   index_table(lines)
}

pub async fn fetch_asset_table(root_url: &str)
      -> ErrStr<Table<NaiveDate,String,USD>> {
   let url = tsv_url(root_url, "assets");
   let lines = fetch_lines(&url).await?;
   fn parse_usd(&str) -> ErrStr<USD> {
      err_or(s.parse(), "Cannot parse USD from {s}")
   }
   ingest(parse_date, parse_str, parse_usd, &lines, "\t")
}

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
   Ok(mk_composition(primary, pivot))
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

/// Fetch the pivots for pivot pool A+B; open pivots are reposed in git
pub async fn fetch_pivots(root_url: &str, primary: &str, pivot: &str,
                          a: &Aliases)
      -> ErrStr<(Vec<Pivot>, Vec<Pivot>, NaiveDate)> {
   let (pri, seggs) = enlowerify(primary, pivot);
   let pool = format!("{pri}+{seggs}");
   let url = open_pivot_path(root_url, &pri, &seggs);
   let lines = fetch_lines(&url).await?;
   parse_pivots(&pool, lines, a)
}

pub fn parse_pivots(pool: &str, lines: Vec<String>, a: &Aliases)
      -> ErrStr<(Vec<Pivot>, Vec<Pivot>, NaiveDate)> {
   let table = index_table(lines)?;

   let hdrs = a.enum_headers(cols(&table));

   let max_date = max_diem(&table, hdrs["opened"], &pool)?;
   let mut acts: Vec<Pivot> = Vec::new();
   let mut pass: Vec<Pivot> = Vec::new();

   for row in table.data {
      let piv = parse_pivot(&hdrs, &row)?;
      if piv.active() {
         acts.push(piv.clone());
      } else {
         pass.push(piv);
      }
   }
   Ok((acts, pass, max_date.clone()))
}

fn enlowerify(primary: &str, pivot: &str) -> (String, String) {
   (primary.to_lowercase(), pivot.to_lowercase())
}

fn enupperify(primary: &str, pivot: &str) -> (String, String) {
   let a = aliases();
   (a.alias(primary), a.alias(pivot))
}

fn max_diem<T>(table: &Table<T, String, String>, ix: usize, pool: &str)
      -> ErrStr<NaiveDate> {
   table.data
        .iter()
        .map(|row| datef(&row[ix]))
        .max()
        .ok_or(format!("No max date for {pool} pivot pool"))
}

/// Filter to only the open pivots for pivot pool A+B
pub async fn fetch_open_pivots(root_url: &str, primary: &str, pivot: &str,
                               a: &Aliases) -> ErrStr<(Vec<Pivot>, NaiveDate)> {
   let (ans, _, max_date) = fetch_pivots(root_url, primary, pivot, a).await?;
   Ok((ans, max_date))
}

async fn fetch_lines(url: &str) -> ErrStr<Vec<String>> {
   let daters = read_rest(url).await?;
   let lines: Vec<String> =
      daters.lines()
            .filter_map(|l| pred(!l.is_empty(), l.to_string()))
            .collect();
   Ok(lines)
}

/// fetch the quotes for date; historical quote-data is reposed in git
pub async fn fetch_quotes(date: &NaiveDate) -> ErrStr<Quotes> {
   let lines = fetch_lines(&quotes_url()).await?;
   let body: Vec<String> = tail(&lines);
   fn capitalize(s: &str) -> ErrStr<String> { Ok(s.to_uppercase()) }
   let table = ingest(parse_date, capitalize, parse_str, &body, ",")?;
   if let Some(quotes_row) = row(&table, date) {
      let mut quotes = HashMap::new();
      let hdrs = cols(&table);
      for (n, h) in hdrs.iter().enumerate() {
         let qt: f32 = parse_num(&quotes_row[n])?;
         quotes.insert(h.clone(), qt);
      }
      Ok(mk_quotes(date.clone(), quotes))
   } else {
      Err(format!("Unable to find quotes for date {date}"))
   }
}

pub mod functional_tests {
   use super::*;
   use book::{
      csv_utils::{CsvWriter,print_as_tsv},
      date_utils::yesterday,
      utils::get_env
   };

   pub fn marshall() -> ErrStr<(String, Aliases)> {
      let root_url = get_env("PIVOT_URL")?;
      let a = aliases();
      Ok((root_url, a))
   }

   pub async fn btc_eth_pivots()
         -> ErrStr<(Vec<Pivot>, Vec<Pivot>, NaiveDate)> {
      let (root_url, a) = marshall()?;
      fetch_pivots(&root_url, "btc", "eth", &a).await
   }

   async fn run_fetch_wallets() -> ErrStr<usize> {
      println!("fetchers::fetch_wallets functional test\n");
      let (root_url, _aliases) = marshall()?;
      let wallets = fetch_wallets(&root_url).await?;
      println!("The wallets for {root_url} are:\n\n{}\n", wallets.as_csv());
      println!("fetchers::fetch_wallets...ok");
      Ok(1)
   }

   async fn run_fetch_asset_table() -> ErrStr<usize> {
      println!("fetchers::fetch_asset_table functional test\n");
      let (root_url, _aliases) = marshall()?;
      let wallets = fetch_wallets(&root_url).await?;
      println!("The wallets for {root_url} are:\n\n{}\n", wallets.as_csv());
      println!("fetchers::fetch_wallets...ok");
      Ok(1)
   }

   async fn run_fetch_assets() -> ErrStr<usize> {
      println!("fetchers::fetch_assets functional test\n");
      let (root_url, a) = marshall()?;
      let assets = fetch_assets(&root_url, "btc", "eth", &a).await?;
      println!("The assets for BTC+ETH are:\n\n{}\n", assets.as_csv());
      println!("fetchers::fetch_assets...ok");
      Ok(1)
   }

   fn print_rows<T:CsvWriter>(title: &str, rows: &[T]) {
      println!("\n{title}:\n");
      for row in rows { print_as_tsv(&row.as_csv()); }
   }

   async fn run_fetch_pivots() -> ErrStr<usize> {
      println!("fetch_pivots functional test\n");
      let (opn, cls, mx) = btc_eth_pivots().await?;
      print_rows("Open pivots", &opn);
      print_rows("Close pivots", &cls);
      println!("\nmax_date is {mx}\n");
      println!("fetch_pivots...ok");
      Ok(1)
   }

   async fn run_fetch_quotes() -> ErrStr<usize> {
      println!("fetch_quotes functional test\n");
      let qts = fetch_quotes(&yesterday()).await?;
      println!("Quotes are:\n{}", qts.as_table().as_csv());
      println!("fetch_quotes...ok");
      Ok(1)
   }

   pub async fn runoff() -> ErrStr<usize> {
      println!("\nfetchers functional tests\n");
      let a = run_fetch_assets().await?;
      let p = run_fetch_pivots().await?;
      let q = run_fetch_quotes().await?;
      let w = run_fetch_wallets().await?;
      Ok(a+p+q+w)
   }
}

#[cfg(test)]
mod tests {
   use std::iter::once;
   use super::*;
   use super::functional_tests::{marshall,btc_eth_pivots};
   use crate::tables::{c2t,csv2tsv};
   use book::{ csv_utils::CsvHeader, date_utils::{yesterday,tomorrow} };

   #[tokio::test]
   async fn test_fetch_wallets_ok() -> ErrStr<()> {
      let (root_url, _aliases) = marshall()?;
      let ans = fetch_wallets(&root_url).await;
      assert!(ans.is_ok());
      Ok(())
   }

   #[tokio::test]
   async fn test_fetch_wallets_table_is_full() -> ErrStr<()> {
      let (root_url, _aliases) = marshall()?;
      let ans = fetch_wallets(&root_url).await?;
      assert!(!ans.data.is_empty());
      Ok(())
   }

   #[tokio::test]
   async fn test_fetch_lines_ok() {
      let ans = fetch_lines(&quotes_url()).await;
      assert!(ans.is_ok());
   }

   #[tokio::test]
   async fn test_fetch_lines() -> ErrStr<()> {
      let ans = fetch_lines(&quotes_url()).await?;
      assert!(!ans.is_empty());
      Ok(())
   }

   #[tokio::test]
   async fn fail_fetch_lines() {
      let ans = fetch_lines("READYOU.md").await;
      assert!(ans.is_err());
   }

   #[tokio::test]
   async fn test_fetch_quotes_ok() {
      let yday = yesterday();
      let ans = fetch_quotes(&yday).await;
      assert!(ans.is_ok());
   }

   #[tokio::test]
   async fn test_fetch_quotes() -> ErrStr<()> {
      let yday = yesterday();
      let ans = fetch_quotes(&yday).await?;
      assert!(ans.lookup(&"BTC".to_string())? > 0.0);
      Ok(())
   }

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

   #[tokio::test]
   async fn test_fetch_pivots_ok() -> ErrStr<()> {
      let mb_opns = btc_eth_pivots().await;
      assert!(mb_opns.is_ok());
      Ok(())
   }

   #[tokio::test]
   async fn test_fetch_pivots() -> ErrStr<()> {
      let (opns, cls, mx) = btc_eth_pivots().await?;
      assert!(!opns.is_empty());
      assert!(!cls.is_empty());
      assert!(tomorrow() > mx);
      Ok(())
   }

   fn pivots_to_tsv(pool: &str, opns: &Vec<Pivot>, cls: &Vec<Pivot>)
         -> ErrStr<Vec<String>> {
      let uno =
         opns.first().or(cls.first())
             .unwrap_or_else(|| panic!("{pool} does not have any pivots!"))
             .header();
      let hdr = c2t(&uno);
      let ops0: Vec<String> = opns.into_iter().map(csv2tsv).collect();
      let cls0: Vec<String> = cls.into_iter().map(csv2tsv).collect();
      Ok(once(hdr).chain(ops0.into_iter().chain(cls0.into_iter())).collect())
   }

   async fn btc_eth_pool_as_tsv() -> ErrStr<Vec<String>> {
      let (opns, cls, _mx) = btc_eth_pivots().await?;
      pivots_to_tsv("BTC+ETH", &opns, &cls)
   }

   #[tokio::test]
   async fn test_pivots_as_tsv_ok() -> ErrStr<()> {
      let tsv = btc_eth_pool_as_tsv().await;
      assert!(tsv.is_ok());
      Ok(())
   }

   #[tokio::test]
   async fn test_reparse_pivots_ok() -> ErrStr<()> {
      let tsv = btc_eth_pool_as_tsv().await?;
      let a = aliases();
      let ans = parse_pivots("BTC+ETH", tsv, &a);
      assert!(ans.is_ok());
      Ok(())
   }

   #[tokio::test]
   async fn test_reparse_pivots() -> ErrStr<()> {
      let tsv = btc_eth_pool_as_tsv().await?;
      let a = aliases();
      let (o,c,m) = parse_pivots("BTC+ETH", tsv, &a)?;
      let (opns, cls, mx) = btc_eth_pivots().await?;
      assert_eq!(opns.len(), o.len());
      assert_eq!(cls.len(), c.len());
      assert_eq!(mx, m);
      Ok(())
   }
}


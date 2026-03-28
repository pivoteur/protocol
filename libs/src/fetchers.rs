/// fetch data from REST endpoints

use std::collections::HashMap;

use chrono::NaiveDate;

use book::{
   currency::usd::USD,
   date_utils::{parse_date,datef},
   err_utils::{err_or,ErrStr},
   list_utils::tail,
   num_utils::{parse_num,parse_commaless},
   parse_utils::{parse_str,parse_usd},
   rest_utils::read_rest,
   string_utils::to_string,
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
      util::{Token,Blockchain,TVLs}
   }
};

pub async fn fetch_wallets(root_url: &str) -> ErrStr<IxTable> {
   let url = tsv_url(root_url, "wallets");
   let lines = fetch_lines(&url).await?;
   index_table(lines)
}

pub async fn fetch_asset_table_tvls(root_url: &str) -> ErrStr<TVLs> {
   let url = tsv_url(root_url, "assets");
   let lines = fetch_lines(&url).await?;
   let hdrs: Vec<Token> =
      lines.first().unwrap().split("\t").map(to_string).collect();
   let first_line: Vec<String> =
      tail(&lines).first().unwrap().split("\t").map(to_string).collect();
   let amts: Vec<USD> =
      tail(&first_line).iter().filter_map(|m| parse_usd(m).ok()).collect();
   let ans: TVLs = tail(&hdrs).into_iter().zip(amts.into_iter()).collect();
   Ok(ans)
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

// ----- TESTS -------------------------------------------------------

#[cfg(not(tarpaulin_include))]
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

   async fn run_fetch_asset_table_tvls() -> ErrStr<usize> {
      println!("fetchers::fetch_asset_table_tvls functional test\n");
      let (root_url, _aliases) = marshall()?;
      let tvls = fetch_asset_table_tvls(&root_url).await?;
      println!("The tvls for the assets {root_url} are:\n\n{tvls:?}\n");
      println!("fetchers::fetch_asset_table_tvls...ok");
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
      let fat = run_fetch_asset_table_tvls().await?;
          // fat: 'fetch_asset_table' ... of course!
      Ok(a+p+q+w+fat)
   }
}

#[cfg(test)]
mod tests {
   use std::iter::once;
   use super::*;
   use super::functional_tests::{marshall,btc_eth_pivots};
   use crate::tables::{c2t,csv2tsv};
   use book::{
      currency::usd::mk_usd,
      csv_utils::CsvHeader,
      date_utils::{yesterday,tomorrow},
      table_utils::val
   };

   #[tokio::test]
   async fn test_fetch_wallets_ok() -> ErrStr<()> {
      let (root_url, _aliases) = marshall()?;
      let ans = fetch_wallets(&root_url).await;
      assert!(ans.is_ok());
      Ok(())
   }

   #[tokio::test]
   async fn test_fetch_wallets_table_data() -> ErrStr<()> {
      let (root_url, _aliases) = marshall()?;
      let ans = fetch_wallets(&root_url).await?;
      assert!(!ans.data.is_empty());
      Ok(())
   }

   fn subset_of_asset_table() -> Vec<String> {
      let raw_table = 
"date	ADA	BTC	ETH	BNB	DOGE	AVAX	HBAR	LTC	LINK	QI	UNDEAD	stable	liquidity pools
2026-03-25	$7,338.70	$87,778.18	$52,697.63	$0.00	$0.00	$16,688.94	$7,007.03	$0.00	$0.00	$0.00	$72,653.56	-$25,635.92	$18,008.40
2026-03-24	$7,165.09	$87,701.80	$52,215.07	$0.00	$0.00	$16,562.41	$6,937.17	$0.00	$0.00	$0.00	$72,553.75	-$25,635.92	$17,957.62
2026-03-23	$6,735.65	$83,993.66	$49,271.17	$0.00	$0.00	$15,496.40	$6,588.36	$0.00	$0.00	$0.00	$73,211.17	-$25,635.92	$17,767.80
2026-03-22	$6,929.07	$84,560.36	$50,309.74	$0.00	$0.00	$15,782.90	$6,653.21	$0.00	$0.00	$0.00	$73,170.25	-$25,635.92	$17,836.67
2026-03-21	$7,149.90	$87,159.75	$52,176.41	$0.00	$0.00	$16,391.29	$6,837.82	$0.00	$0.00	$0.00	$81,113.85	-$25,635.92	$19,083.35
2026-03-20	$7,240.63	$86,734.73	$51,733.48	$0.00	$0.00	$16,402.44	$6,890.66	$0.00	$0.00	$0.00	$80,181.29	-$25,635.92	$18,956.66
2026-03-19	$7,229.82	$85,739.32	$52,059.70	$0.00	$0.00	$16,362.45	$6,831.71	$0.00	$0.00	$0.00	$80,746.88	-$25,635.92	$19,024.19
2026-03-18	$7,472.98	$89,303.32	$54,226.97	$0.00	$0.00	$16,873.15	$7,119.13	$0.00	$0.00	$0.00	$81,345.75	-$25,635.92	$19,245.12
2026-03-17	$7,689.12	$90,991.08	$56,074.31	$0.00	$0.00	$17,622.49	$7,259.15	$0.00	$0.00	$0.00	$84,940.30	-$25,635.92	$19,635.28";
      let assets: Vec<String> = raw_table.lines().map(to_string).collect();
      assets
   }

   #[test]
   fn test_fetch_subset_of_asset_table_ok() {
      let lines = subset_of_asset_table();
      let assets = ingest(parse_date, parse_str, parse_usd, &lines, "\t");
      assert!(assets.is_ok());
   }

   #[test]
   fn test_fetch_subset_of_asset_table_datum() -> ErrStr<()> {
      fn s(a: &str) -> String { a.to_string() }
      let lines = subset_of_asset_table();
      let assets = ingest(parse_date, parse_str, parse_usd, &lines, "\t")?;
      let stable = mk_usd(-25635.92);
      let tday = parse_date("2026-03-25")?;
      assert_eq!(Some(stable), val(&assets, &tday, &s("stable")));
      let lps = mk_usd(18008.4);
      assert_eq!(Some(lps), val(&assets, &tday, &s("liquidity pools")));
      Ok(())
   }

   #[tokio::test]
   async fn test_fetch_asset_table_tvls_ok() -> ErrStr<()> {
      let (root_url, _aliases) = marshall()?;
      let ans = fetch_asset_table_tvls(&root_url).await;
      assert!(ans.is_ok());
      Ok(())
   }

   #[tokio::test]
   async fn test_fetch_asset_table_tvls_data() -> ErrStr<()> {
      let (root_url, _aliases) = marshall()?;
      let ans = fetch_asset_table_tvls(&root_url).await?;
      assert!(!ans.is_empty());
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


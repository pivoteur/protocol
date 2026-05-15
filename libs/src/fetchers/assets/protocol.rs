use book::{
   currency::usd::USD,
   err_utils::ErrStr,
   list_utils::{tail,filter_map_or},
   parse_utils::parse_usd,
   string_utils::{str2strf,to_string}
};

use crate::{
   fetchers::utils::fetch_lines,
   paths::csv_url,
   types::{
      util::{Token,TVLs}
   }
};

// ----- PROTOCOL ASSETS --------------------------------------------

pub async fn fetch_asset_table_tvls(root_url: &str) -> ErrStr<TVLs> {
   let url = csv_url(root_url, "assets");
   let lines = fetch_lines(&url).await?;
   let hdrs: Vec<Token> =
      lines.first().unwrap().split(",").map(to_string).collect();
   let first_line: Vec<String> =
      tail(&lines).first().unwrap().split(",").map(to_string).collect();
   let amts: Vec<USD> =
      filter_map_or(str2strf(parse_usd), tail(&first_line))?;
   let ans: TVLs = tail(&hdrs).into_iter().zip(amts.into_iter()).collect();
   Ok(ans)
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{
      create_testing,
      utils::now
   };
   use crate::fetchers::test_helpers::test_functions::marshall;

   create_testing!("fetchers::assets::protocol");

   run!("fetch_asset_table_tvls", {
      let (root_url, _aliases) = marshall()?;
      let tvls = now(fetch_asset_table_tvls(&root_url))?;
      fn print_pair((tok, p): &(String, USD)) { println!("\t{tok}: {p}"); }
      println!("The tvls for the assets {root_url} are:\n");
      tvls.iter().for_each(print_pair);
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use crate::fetchers::test_helpers::test_functions::marshall;
   use book::{
      currency::usd::mk_usd,
      date_utils::parse_date,
      parse_utils::parse_str,
      table_utils::{ingest,val}
   };
      
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
}

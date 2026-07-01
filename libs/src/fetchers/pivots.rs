use chrono::NaiveDate;

use book::{
   date_utils::datef,
   err_utils::ErrStr,
   table_utils::{Table,cols},
   tuple_utils::{Partition,fst}
};

use super::utils::fetch_lines;

use crate::{
   paths::open_pivot_path,
   tables::index_table,
   types::{ aliases::Aliases, pivots::{Pivot,parse_pivot}, pools::Pool }
};

// ----- PIVOTS -------------------------------------------------------

/// Fetch the pivots for pivot pool A+B; open pivots are reposed in git
pub async fn fetch_pivots(root_url: &str, pool: &Pool, a: &Aliases)
      -> ErrStr<(Partition<Pivot>, NaiveDate)> {
   let url = open_pivot_path(root_url, pool);
   let lines = fetch_lines(&url).await?;
   parse_pivots(pool, lines, a)
}

pub fn parse_pivots(pool: &Pool, lines: Vec<String>, a: &Aliases)
      -> ErrStr<(Partition<Pivot>, NaiveDate)> {
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
   Ok(((acts, pass), max_date.clone()))
}

fn max_diem<T>(table: &Table<T, String, String>, ix: usize, pool: &Pool)
      -> ErrStr<NaiveDate> {
   table.data
        .iter()
        .map(|row| datef(&row[ix]))
        .max()
        .ok_or(format!("No max date for {pool} pivot pool"))
}

/// Filter to only the open pivots for pivot pool A+B
pub async fn fetch_open_pivots(root_url: &str, pool: &Pool, a: &Aliases)
      -> ErrStr<(Vec<Pivot>, NaiveDate)> {
   let (part, max_date) = fetch_pivots(root_url, pool, a).await?;
   Ok((fst(part), max_date))
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, list_utils::take, utils::now };
   use crate::{
      fetchers::test_helpers::test_functions::btc_eth_pivots,
      reports::print_tsv_table_d
   };

   create_testing!("fetchers::pivots");

   run!("fetch_pivots", {
      let ((opn, cls), mx) = now(btc_eth_pivots())?;
      print_tsv_table_d("Open pivots", &take(3, &opn), true);
      print_tsv_table_d("Close pivots", &take(3, &cls), true);
      println!("\nmax_date is {mx}\n");
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use std::iter::once;
   use super::*;
   use crate::{
      fetchers::test_helpers::test_functions::btc_eth_pivots,
      tables::{c2t,csv2tsv},
      types::{aliases::aliases,pools::pool_from_str}
   };
   use book::{ csv_utils::CsvHeader, date_utils::tomorrow };

   #[tokio::test]
   async fn test_fetch_pivots_ok() -> ErrStr<()> {
      let mb_opns = btc_eth_pivots().await;
      assert!(mb_opns.is_ok());
      let ((opns, cls), mx) = mb_opns?;
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
      let ((opns, cls), _mx) = btc_eth_pivots().await?;
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
      let pool = pool_from_str("BTC+ETH")?;
      let ans = parse_pivots(&pool, tsv, &a);
      assert!(ans.is_ok());
      Ok(())
   }

   #[tokio::test]
   async fn test_reparse_pivots() -> ErrStr<()> {
      let tsv = btc_eth_pool_as_tsv().await?;
      let a = aliases();
      let pool = pool_from_str("btc-eth")?;
      let ((o,c),m) = parse_pivots(&pool, tsv, &a)?;
      let ((opns, cls), mx) = btc_eth_pivots().await?;
      assert_eq!(opns.len(), o.len());
      assert_eq!(cls.len(), c.len());
      assert_eq!(mx, m);
      Ok(())
   }
}

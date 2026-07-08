use chrono::NaiveDate;
use clap::Parser;

use book::{
   parse_args_add_banner,
   cli_utils::add_banner,
   csv_utils::{CsvHeader,print_csv},
   err_utils::ErrStr,
   string_utils::UppercaseString,
   utils::get_env
};

use libs::{
   fetchers::{ pivots::fetch_pivots, quotes::fetch_quotes},
   types::{
      pivots::{ Pivot, next_close_id },
      pools::{Pool,mk_pool},
      proposals::proposes::{propose,Propose}
   }
};

struct Report {
   pool: Pool,
   opens: usize,
   date: NaiveDate,
   props: Vec<Propose>,
   max_date: NaiveDate
}
fn mk_report(pool: &Pool, opns: &[Pivot], date: &NaiveDate, props: &[Propose],
             max_date: &NaiveDate) -> ErrStr<Report> {
   Ok(Report { pool: pool.clone(),
               opens: opns.len(),
               date: date.clone(),
               props: props.to_vec(),
               max_date: max_date.clone() })
}

async fn compute_closes(root_url: &str, pool: &Pool, date: &NaiveDate,
                        debug: bool) -> ErrStr<Report> {
   let quotes = fetch_quotes(&date).await?;
   let a = &quotes.aliases;
   let ((opns, cls), max_date) = fetch_pivots(root_url, pool, a, debug).await?;
   let mut next_close = next_close_id(&cls);
   let proposer = propose(&quotes);
   let mut props = Vec::new();

   if debug { println!("Processing open pivots for {pool} pivot pool"); }
   for h in &opns {
      let hs = vec![h.clone()];
      if debug { println!("\tprocessing {pool} open pivot #{}", h.index()); }
      if let Some((prop, next_next)) = proposer((hs, next_close), debug)? {
         props.push(prop);
         next_close = next_next;
      }
   }

   mk_report(pool, &opns, date, &props, &max_date)
}

fn report_proposes(rpt: Report) -> ErrStr<()> {
   let mut print_header: bool = true;
   let header = rpt.pool.pool_name();
   let pool = format!("{header} pivot pool");
   let len = rpt.opens;

   println!("{header}\n");
   println!("There are {len} open pivots for the {pool}.");
   println!("The last entry is on {}.", rpt.max_date);
   println!("Recommendations are made for token quotes on {}.\n", rpt.date);

   for prop in rpt.props {
      if print_header {
         println!("{}",prop.header());
         print_header = false;
      }
      print_csv(&prop);
   }
   
   let no_close_pivots = print_header;
   if no_close_pivots {
      println!("No close pivot recommendations for {pool}.");
   }
   Ok(())
}

fn print_heading() { println!("chihuahua, version: 1.02\n"); }

/// Proposes close pivots for a selected pivot pool
///
/// The pivot pools are reposed (in git, currently)
/// Open pivots are stored as raw-CSV files in git at protocol
#[derive(Debug, Parser)]
struct Args {

   /// Protocol to analyze for close pivot calls, e.g.: PIVOT
   protocol: UppercaseString,

   /// Primary asset of pivot pool, e.g.: BTC
   primary: String,

   /// Pivot asset of pivot pool, e.g.: ETH
   pivot: String,

   /// Date to analyze close pivots, e.g.: $LE_DATE
   date: NaiveDate,

   /// Print debugging information
   #[arg(short, long)]
   debug: bool
}

pub async fn runoff_get_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   let pool = mk_pool(&args.primary, &args.pivot);
   report_calls(&args.protocol, &pool, &args.date, args.debug).await
}

async fn report_calls(protocol: &str, pool: &Pool, date: &NaiveDate,
                      debug: bool) -> ErrStr<()> {
   let root_url = get_env(&format!("{}_URL", protocol))?;
   print_heading();
   let report = compute_closes(&root_url, pool, date, debug).await?;
   report_proposes(report)
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, date_utils::yesterday, utils::now };

   create_testing!("quiz02::b_compute_close");

   run!("report_calls", {
      let pool = mk_pool("avax", "undead");
      now(report_calls("PIVOT", &pool, &yesterday(), true))?;
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {

   use super::*;
   use book::{ date_utils::{ parse_date, yesterday }, utils::get_env };
   use libs::types::pools::pool_from_str;

   async fn compute_test_closes() -> ErrStr<Report> {
      let pivot_url = get_env("PIVOT_URL")?;
      let pool = pool_from_str("avax-undead")?;
      compute_closes(&pivot_url, &pool, &yesterday(), true).await
   }

   #[tokio::test]
   async fn test_compute_closes_ok() {
      let report = compute_test_closes().await;
      assert!(report.is_ok());
   }

   #[tokio::test]
   async fn test_compute_closes_last_entry() -> ErrStr<()> {
      let report = compute_test_closes().await?;
      let new_year = parse_date("2026-01-01")?;
      assert!(report.max_date > new_year);
      Ok(())
   }

   #[tokio::test]
   async fn test_proposes_subset_of_open_pivots() -> ErrStr<()> {
      let report = compute_test_closes().await?;
      assert!(report.opens >= report.props.len());
      Ok(())
   }
}


use chrono::NaiveDate;

use book::{
   csv_utils::{CsvHeader,print_csv},
   date_utils::parse_date,
   err_utils::ErrStr
};

use libs::{
   fetchers::{fetch_pivots,fetch_quotes},
   types::pivots::{next_close_id,propose,Propose}
};

struct Report {
   primary: String,
   pivot: String,
   opens: usize,
   date: NaiveDate,
   props: Vec<Propose>,
   max_date: NaiveDate
}

async fn compute_closes(root_url: &str, prim: &str, piv: &str, date: NaiveDate)
      -> ErrStr<Report> {
   let (opens, closes, max_date) = fetch_pivots(root_url, prim, piv).await?;
   let quotes = fetch_quotes(&date).await?;
   let mut next_close = next_close_id(&closes);
   let proposer = propose(&quotes);
   let mut props = Vec::new();

   for h in &opens {
      let hs = vec![h.clone()];
      if let Some((prop, next_next)) = proposer((hs, next_close))? {
         props.push(prop);
         next_close = next_next;
      }
   }

   Ok(Report { primary: prim.to_string(),
               pivot: piv.to_string(),
               opens: opens.len(),
               date, props, max_date })
}

fn report_proposes(rpt: Report) {
   let mut print_header: bool = true;
   let cap_prim = rpt.primary.to_uppercase();
   let cap_piv = rpt.pivot.to_uppercase();
   let header = format!("{cap_prim}+{cap_piv}");
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
}

fn app_name() -> String { "chihuahua".to_string() }
fn version() -> String { "1.01".to_string() }
fn print_heading() { println!("{}, version: {}\n", app_name(), version()); }

fn usage() -> ErrStr<()> {
   println!("Usage:

	$ {} <root URL> <primary asset> <pivot asset> <date>

Proposes close pivots for the <prim>+<piv> pivot pool for <date>.
The pivot pools are reposed (in git, currently) at <root URL>.

Open pivots are stored as raw-CSV files in git at protocol <root URL>.
", app_name());
   Err("Needs <root URL> <primary> <pivot> <date> arguments".to_string())
}

pub mod functional_tests {

   use super::*;

   use book::{
      string_utils::to_string,
      utils::{get_args,get_env}
   };

   pub async fn runoff_get_args() -> ErrStr<()> {
      let args = get_args();
      do_it(args).await
   }

   async fn do_it(args: Vec<String>) -> ErrStr<()> {
      print_heading();
      if let [root_url, prim, piv, date] = args.as_slice() {
         let dt = parse_date(&date)?;
         let report = compute_closes(root_url, prim, piv, dt).await?;
         report_proposes(report);
         Ok(())
      } else {
         usage()
      }
   }

   pub async fn runoff() -> ErrStr<usize> {

      println!("\nquiz02: b_compute_close functional test\n");

      let pivot_url = get_env("PIVOT_URL")?;
      let args: Vec<String> = [&pivot_url, "AVAX", "UNDEAD", "2026-01-25"]
         .into_iter().map(to_string).collect();
      match do_it(args).await { Ok(()) => Ok(1), Err(x) => Err(x) }
   }
}

#[cfg(test)]
mod tests {

   use super::*;

   use book::utils::get_env;

   async fn compute_test_closes() -> ErrStr<Report> {
      let dt = parse_date("2026-01-25")?;
      let pivot_url = get_env("PIVOT_URL")?;
      compute_closes(&pivot_url, "AVAX", "UNDEAD", dt).await
   }

   #[tokio::test]
   async fn test_compute_closes_ok() -> ErrStr<()> {
      let report = compute_test_closes().await;
      assert!(report.is_ok());
      Ok(())
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


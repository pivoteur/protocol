use chrono::NaiveDate;

use book::{
//   csv_utils::print_as_tsv,
   date_utils::parse_date,
   err_utils::ErrStr,
   utils::get_args
};

use libs::{
   fetchers::{fetch_pivots,fetch_quotes},
   reports::{preamble,print_table},
   types::pivots::{partition_on,next_close_id,propose}
};

#[tokio::main]
async fn main() -> ErrStr<()> {
   if let [root_url, prim, piv, date] = get_args().as_slice() {
      let dt = parse_date(&date)?;
      do_it(root_url, prim, piv, dt).await
   } else {
      usage()
   }
}

async fn do_it(root_url: &str, prim: &str, piv: &str, date: NaiveDate)
      -> ErrStr<()> {
   let (opens, closes, max_date) = fetch_pivots(root_url, prim, piv).await?;
   let quotes = fetch_quotes(&date).await?;
   let next_close = next_close_id(&closes);
   preamble(prim, piv, opens.len(), &max_date, &date);
   let proposer = propose(&quotes);

   let (lefts, rights) = partition_on(prim, opens);
   let mut props = Vec::new();
   let follow = if let Some((prop, nxt)) = proposer((lefts, next_close))? {
      props.push(prop);
      nxt
   } else {
      next_close
   };
   if let Some((prop, _)) = proposer((rights, follow))? {
      props.push(prop);
   }
   fn printer(s: &String) { println!("{s}"); }
   print_table(printer, "No close pivots", &props);
   Ok(())
}

fn usage() -> ErrStr<()> {
   println!("Usage:

	$ cargo run <root URL> <primary asset> <pivot asset> <date>

Partitions open pivots then aggregates proposed close pivots.

The pivot pools are reposed (in git, currently) at <root URL>.

Open pivots are stored as raw-CSV files in git at protocol <root URL>.
");
   Err("Needs <root URL> <primary> <pivot> <date> arguments".to_string())
}

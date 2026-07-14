use chrono::NaiveDate;
use clap::Parser;

use book::{
   parse_args_add_banner,
   cli_utils::add_banner,
   currency::usd::USD,
   csv_utils::{CsvWriter,CsvHeader},
   err_utils::ErrStr,
   string_utils::UppercaseString,
   tuple_utils::Partition,
   utils::get_env
};

use libs::{
   collections::assets::mk_assets,
   fetchers::{ quotes::fetch_quotes, pivots::fetch_pivots},
   paths::pivot_pool_from_file,
   processors::virtuals::recompute_pivot,
   reports::{total_line,print_tsv_table_d},
   types::{
      comps::Composition,
      measurable::{Measurable,tvl},
      pivots::opens::Pivot,
      pools::Pool,
      quotes::Quotes
   }
};

fn partition_virtual_pivots(all_opns: Vec<Pivot>) -> Partition<Pivot> {
   all_opns.into_iter().partition(Pivot::is_virtual)
}

fn aggregate_virtual_pivots(virts: &[Pivot], quotes: &Quotes, pool: &Pool)
      -> ErrStr<Composition> {
   let mut asts = mk_assets();
   virts.iter().for_each(|v| {
      let asset = v.committed()
          .unwrap_or_else(|err|
              panic!("unable to process pivot {}, err: {err:?}", v.as_csv()));
      asts.add(asset);
   });

   asts.as_composition(pool, quotes)
}

fn tvls<T:Measurable>(rows: &[T]) -> USD { rows.iter().map(tvl).sum() }

async fn update_virtual_pivots(protocol: &str, date: &NaiveDate, path: &str,
                             debug: bool) -> ErrStr<()> {
   let pool = pivot_pool_from_file(path)?;
   let root_url = get_env(&format!("{protocol}_URL"))?;
   let quotes = fetch_quotes(&date).await?;
   let truz = &quotes.aliases;
   let (pivots, _mx) = fetch_pivots(&root_url, &pool, truz, debug).await?;
   let (all_opns, cls) = pivots;
   let (virts, opns) = partition_virtual_pivots(all_opns);
   let pool_name = pool.pool_name();

   if debug {
      if !virts.is_empty() {
         let agg = aggregate_virtual_pivots(&virts, &quotes, &pool)?;
         report_on_assets(&[agg], &virts);
      } else {
         println!("Pivot pool {pool_name} has no virtual pivots.");
      }
   }

   let comp = recompute_pivot(&quotes, debug);
   let new_virts: Vec<Pivot> =
      virts.into_iter().filter_map(|v| comp(v).ok()).collect();
   let mut new_opens: Vec<Pivot> = 
      opns.into_iter().chain(cls.into_iter()
                      .chain(new_virts.into_iter()))
                      .collect();
   new_opens.sort_by(|a,b| a.index().cmp(&b.index()));
   tabl(&format!("{pool_name} pivots"), &new_opens, 3, debug);
   Ok(())
}

fn report_on_assets(pools: &[Composition], virts: &[Pivot]) {
   tabl("Virtual Pivot Assets", pools, 3, true);
   tabl("Virtual pivots", virts, 3, true);
}

fn tabl<T:CsvWriter + CsvHeader + Measurable>
      (title: &str, rows: &[T], offset: usize, debug: bool) {
   let skip = if let Some(a_row) = rows.first() { a_row.ncols() } else {
      panic!("Portfolio has no pivot pools!")
   } - offset;
   print_tsv_table_d(title, rows, debug);
   if debug { total_line(skip, " ,total", &tvls(rows)); }
}

/// Computes assets committed to virtual pivots.
#[derive(Debug, Parser)]
#[command(name = "virtsz")]
#[command(version = "2.05")]
struct Args {
   /// Protocol to compute assets committed to virtual pivots, e.g.: PIVOT
   protocol: UppercaseString,

   /// date on which to compute assets committed to virtual pivots
   date: NaiveDate,

   /// path to pivot pool to analyze, e.g. data/pivots/open/raw/btc-eth.tsv
   path: String,

   /// prints debugging information
   #[arg(short, long)]
   debug: bool
}

pub async fn runoff_with_args() -> ErrStr<()> {
   let args = parse_args_add_banner!(Args);
   update_virtual_pivots(&args.protocol, &args.date, &args.path,
                         args.debug).await
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod test_data {
   use super::*;
   use libs::fetchers::test_helpers::test_functions::btc_eth_pivots;
   use book::tuple_utils::fst;

   pub async fn virts_n_opns() -> ErrStr<(Vec<Pivot>, Partition<Pivot>)> {
      let (pivots, _mx) = btc_eth_pivots().await?;
      let all_opns = fst(pivots);
      let (virts, opns) = partition_virtual_pivots(all_opns.clone());
      Ok((all_opns, (virts, opns)))
   }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use super::test_data::virts_n_opns;
   use paste::paste;
   use libs::{
      paths::paths_test_helpers::path_to_btc_eth_pivot_pool,
      types::pools::mk_pool
   };
   use book::{
      create_testing,
      date_utils::yesterday,
      utils::now
   };

   create_testing!("quiz07::b_virtual");

   run!("update_virtual_pivots", {
      let path = path_to_btc_eth_pivot_pool();
      let _ = now(update_virtual_pivots("pivot", &yesterday(), &path, true));
   });

   run!("aggregate_virtual_pivots", {
      let (_, (virts, _)) = now(virts_n_opns())?;
      assert!(!virts.is_empty());
      let yday = yesterday();
      let qt = now(fetch_quotes(&yday))?;
      let pool = mk_pool("BTC", "ETH");
      let comp = aggregate_virtual_pivots(&virts, &qt, &pool)?;
      println!("The virtual assets for {pool} are:
{}", comp.as_csv());
   });
}

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod tests {
   use super::*;
   use super::test_data::virts_n_opns;
   use book::num::estimate::mk_estimate;

   #[tokio::test] async fn test_partition_virtual_pivots() -> ErrStr<()> {
      let (all_opns, (virts, opns)) = virts_n_opns().await?;
      assert_eq!(all_opns.len(), virts.len() + opns.len());
      let tvlsz = tvls(&virts) + tvls(&opns);
      let est = mk_estimate(tvls(&all_opns).amount());
      est.is(tvlsz.amount())
   }
}

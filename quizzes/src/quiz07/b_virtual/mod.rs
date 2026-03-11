use book::{
   currency::usd::{USD,mk_usd},
   csv_utils::{CsvWriter,CsvHeader},
   date_utils::parse_date,
   err_utils::ErrStr,
   tuple_utils::Partition,
   utils::get_env
};

use libs::{
   collections::assets::{Assets,mk_assets,assets_by_price},
   fetchers::{fetch_quotes,fetch_pivots},
   reports::{header,total_line,print_table_d},
   types::{
      assets::{Asset,mk_asset},
      comps::{Composition,mk_composition},
      measurable::{Measurable,tvl},
      pivots::Pivot,
      quotes::Quotes,
      util::{Blockchain,Token,Pool,mk_pool}
   }
};

fn version() -> String { "2.01".to_string() }
fn app_name() -> String { "virtsz".to_string() }

fn partition_virtual_pivots(all_opns: Vec<Pivot>) -> Partition<Pivot> {
   all_opns.into_iter().partition(Pivot::is_virtual)
}

fn aggregate_virtual_pivots(virts: &[Pivot], blk: &Blockchain,
                            quotes: &Quotes, pool: &Pool) -> ErrStr<Assets> {
   let (pri, piv) = pool;
   let mut asts = mk_assets();
   virts.iter().for_each(|v| asts.add(v.committed()));

/* 4 scenarii: 

1. no matches, no virtual pivots
2. 1 match on primary
3. 1 match on pivot
4. 2 matches: primary, pivot

so, you know: handle those.
*/

   fn nonce<'a>(b: &'a Blockchain, q: &'a Quotes)
         -> impl Fn(&'a Token) -> ErrStr<Asset> {
      move |tok| {
         let qt = q.lookup(tok)?;
         Ok(mk_asset(&(b.clone(), tok.clone()), 0.0, &mk_usd(qt), &q.date))
      }
   }
   let zed = nonce(&blk, &quotes);
   asts.add(zed(&pri)?);
   asts.add(zed(&piv)?);
   Ok(asts)
}

fn tvls<T:Measurable>(rows: &[T]) -> USD { rows.iter().map(tvl).sum() }

async fn show_virtual_pivots(protocol: &str, dt: &str, p0: &Token, p1: &Token,
                             debug: bool) -> ErrStr<()> {
   let pri = p0.to_uppercase();
   let piv = p1.to_uppercase();
   let pool_name = header(&pri, &piv);
   let pool = mk_pool(&pri, &piv);
   let auth = protocol.to_uppercase();
   let root_url = get_env(&format!("{auth}_URL"))?;
   let date = parse_date(&dt)?;
   let quotes = fetch_quotes(&date).await?;
   let truz = &quotes.aliases;
   let (all_opns, _cls, _mx) = fetch_pivots(&root_url, &pri, &piv, truz).await?;
   let (virts, _opns) = partition_virtual_pivots(all_opns);

   if !virts.is_empty() {
      let blk = virts.first().unwrap().blockchain();
      let asts = aggregate_virtual_pivots(&virts, &blk, &quotes, &pool)?;
      let abp = assets_by_price(&asts);

      if let [pr, pv] = abp.as_slice() {
         let comp = mk_composition(pr.clone(), pv.clone());
         report_on_assets(&[comp], &virts, debug);
      } else {
         panic!("Not two assets in {pool_name} Assets: {:?}", abp)
      }
   } else {
      if debug { println!("Pivot pool {pool_name} has no virtual pivots."); }
   }
   Ok(())
}

fn report_on_assets(pools: &[Composition], virts: &[Pivot], debug: bool) {
   if debug {
      println!("{}, version: {}", app_name(), version());
      tabl("Virtual Pivot Assets", pools, 3, debug);
   }
   tabl("Virtual pivots", virts, 2, debug);
}

fn tabl<T:CsvWriter + CsvHeader + Measurable>
      (title: &str, rows: &[T], offset: usize, debug: bool) {
   let skip = if let Some(a_row) = rows.first() { a_row.ncols() } else {
      panic!("Portfolio has no pivot pools!")
   } - offset;
   print_table_d(title, rows, debug);
   if debug { total_line(skip, " ,total", &tvls(rows)); }
}

fn usage() -> String {
   println!("\n$ ./{} [--debug|-d] <protocol> <date> <prim> <piv>

Computes assets committed to virtual pivots.

where
* -d or --debug, if present, means print debug information
* <protocol> is the protocol, e.g. PIVOT
* <date> to check availability, e.g.: $LE_DATE
* <prim> primary asset, e.g.: BTC
* <piv> pivot asset, e.g.: ETH
", app_name());
   "Needs arguments <protocol> <date> <prim> <piv>".to_string()
}

pub mod functional_tests {
   use super::*;
   use book::{ date_utils::yesterday, list_utils::tail, utils::get_args };

   pub async fn runoff_with_args() -> ErrStr<()> {
      let args = get_args();
      if let Some(arg) = args.first() {
         let (args1, debug) = if arg == "--debug" || arg == "-d" {
            (tail(&args), true)
         } else { (args.clone(), false) };
         if let [protocol, dt, pri, piv] = args1.as_slice() {
            show_virtual_pivots(&protocol, &dt, pri, piv, debug).await
         } else { Err(usage()) }
      } else { Err(usage()) }
   }

   pub async fn runoff() -> ErrStr<usize> {
      let yday = format!("{}", yesterday());
      fn s(t: &str) -> Token { t.to_string() }
      let _ = show_virtual_pivots("pivot", &yday,
                     &s("btc"), &s("eth"), true).await?;
      Ok(1)
   }
}

#[cfg(test)]
mod tests {
   use chrono::NaiveDate;
   use super::*;
   use book::{ date_utils::yesterday, utils::get_env };
   use libs::types::aliases::aliases;

   async fn btc_eth_pivots() -> ErrStr<(Vec<Pivot>, Vec<Pivot>, NaiveDate)> {
      let a = aliases();
      let root_url = get_env("PIVOT_URL")?;
      fetch_pivots(&root_url, "btc", "eth", &a).await
   }

   async fn virts_n_opns() -> ErrStr<(Vec<Pivot>, Partition<Pivot>)> {
      let (all_opns, _cls, _mx) = btc_eth_pivots().await?;
      let (virts, opns) = partition_virtual_pivots(all_opns.clone());
      Ok((all_opns, (virts, opns)))
   }

   #[tokio::test]
   async fn test_partition_virtual_pivots() -> ErrStr<()> {
      let (all_opns, (virts, opns)) = virts_n_opns().await?;
      assert_eq!(all_opns.len(), virts.len() + opns.len());
      assert_eq!(tvls(&all_opns), tvls(&virts) + tvls(&opns));
      Ok(())
   }

   #[tokio::test]
   async fn test_aggregate_virtual_pivots() -> ErrStr<()> {
      let (_, (virts, _)) = virts_n_opns().await?;
      assert!(!virts.is_empty());
      let yday = yesterday();
      let qt = fetch_quotes(&yday).await?;
      let pool = mk_pool("BTC", "ETH");
      let blk = "Avalanche".to_string();
      let assets = aggregate_virtual_pivots(&virts, &blk, &qt, &pool)?;
      let abq = assets_by_price(&assets);
      assert_eq!(2, abq.len());
      Ok(())
   }
}

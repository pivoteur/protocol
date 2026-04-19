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
   paths::pivot_pool_from_file,
   reports::{header,total_line,print_tsv_table_d},
   types::{
      coins::{Coin,mk_coin},
      comps::{Composition,mk_composition},
      measurable::{Measurable,tvl},
      pivots::{Pivot,recompute_pivot},
      quotes::Quotes,
      util::{Blockchain,Token,Pool,mk_pool}
   }
};

fn version() -> String { "2.03".to_string() }
fn app_name() -> String { "virtsz".to_string() }

fn partition_virtual_pivots(all_opns: Vec<Pivot>) -> Partition<Pivot> {
   all_opns.into_iter().partition(Pivot::is_virtual)
}

fn aggregate_virtual_pivots(virts: &[Pivot], blk: &Blockchain,
                            quotes: &Quotes, pool: &Pool) -> ErrStr<Assets> {
   let (pri, piv) = pool;
   let mut asts = mk_assets();
   virts.iter().for_each(|v| {
      let asset = v.committed()
          .unwrap_or_else(|err|
              panic!("unable to process pivot {}, err: {err:?}", v.as_csv()));
      asts.add(asset);
   });

/* 4 scenarii: 

1. no matches, no virtual pivots
2. 1 match on primary
3. 1 match on pivot
4. 2 matches: primary, pivot

so, you know: handle those.
*/

   fn nonce<'a>(b: &'a Blockchain, q: &'a Quotes)
         -> impl Fn(&'a Token) -> ErrStr<Coin> {
      move |tok| {
         let qt = q.lookup(tok)?;
         Ok(mk_coin(&(b.clone(), tok.clone()), 0.0, &mk_usd(qt), &q.date))
      }
   }
   let zed = nonce(&blk, &quotes);
   asts.add(zed(&pri)?);
   asts.add(zed(&piv)?);
   Ok(asts)
}

fn tvls<T:Measurable>(rows: &[T]) -> USD { rows.iter().map(tvl).sum() }

async fn update_virtual_pivots(protocol: &str, dt: &str, path: &str,
                             debug: bool) -> ErrStr<()> {
   let (p0, p1) = pivot_pool_from_file(path)?;
   let pri = p0.to_uppercase();
   let piv = p1.to_uppercase();
   let pool_name = header(&pri, &piv);
   let pool = mk_pool(&pri, &piv);
   let auth = protocol.to_uppercase();
   let root_url = get_env(&format!("{auth}_URL"))?;
   let date = parse_date(&dt)?;
   let quotes = fetch_quotes(&date).await?;
   let truz = &quotes.aliases;
   let (pivots, _mx) = fetch_pivots(&root_url, &pri, &piv, truz).await?;
   let (all_opns, cls) = pivots;
   let (virts, opns) = partition_virtual_pivots(all_opns);

   if debug {
      if !virts.is_empty() {
         let blk = virts.first().unwrap().blockchain();
         let asts = aggregate_virtual_pivots(&virts, &blk, &quotes, &pool)?;
         let abp = assets_by_price(&asts);

         if let [pr, pv] = abp.as_slice() {
            let comp = mk_composition(pr.clone(), pv.clone());
            report_on_assets(&[comp], &virts);
         } else {
            panic!("Not two assets in {pool_name} Assets: {:?}", abp)
         }
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
   println!("{}, version: {}", app_name(), version());
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

fn usage() -> String {
   println!("\n$ ./{} [--debug|-d] <protocol> <date> <path>

Computes assets committed to virtual pivots.

where
* -d or --debug, if present, means print debug information

* <protocol> is the protocol,
         e.g. PIVOT

* <date> to check availability,
         e.g.: $LE_DATE

* <path> to the pivot pool file to process,
         e.g. protocol/data/pivots/open/raw/btc-eth.tsv
", app_name());
   "Needs arguments <protocol> <date> <prim> <piv>".to_string()
}

// ----- TESTS -------------------------------------------------------

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use libs::paths::functional_tests::path_to_btc_eth_pivot_pool;
   use book::{ date_utils::yesterday, list_utils::tail, utils::get_args };

   pub async fn runoff_with_args() -> ErrStr<()> {
      let args = get_args();
      if let Some(arg) = args.first() {
         let (args1, debug) = if arg == "--debug" || arg == "-d" {
            (tail(&args), true)
         } else { (args.clone(), false) };
         if let [protocol, dt, path] = args1.as_slice() {
            update_virtual_pivots(&protocol, &dt, path, debug).await
         } else { Err(usage()) }
      } else { Err(usage()) }
   }

   pub async fn runoff() -> ErrStr<usize> {
      let yday = format!("{}", yesterday());
      let _ = update_virtual_pivots("pivot", &yday,
                     &path_to_btc_eth_pivot_pool(), true).await?;
      Ok(1)
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use book::{ date_utils::yesterday, tuple_utils::fst };
   use libs::fetchers::functional_tests::btc_eth_pivots;

   async fn virts_n_opns() -> ErrStr<(Vec<Pivot>, Partition<Pivot>)> {
      let (pivots, _mx) = btc_eth_pivots().await?;
      let all_opns = fst(pivots);
      let (virts, opns) = partition_virtual_pivots(all_opns.clone());
      Ok((all_opns, (virts, opns)))
   }

   #[tokio::test]
   async fn test_partition_virtual_pivots() -> ErrStr<()> {
      let (all_opns, (virts, opns)) = virts_n_opns().await?;
      assert_eq!(all_opns.len(), virts.len() + opns.len());
      fn around(a: f32, b: f32) -> bool {
         ((a - b) / b).abs() < 0.01
      }
      let tvlsz = tvls(&virts) + tvls(&opns);
      assert!(around(tvls(&all_opns).amount, tvlsz.amount));
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

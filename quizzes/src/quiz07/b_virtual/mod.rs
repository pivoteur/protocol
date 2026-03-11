use chrono::NaiveDate;

use book::{
   currency::usd::mk_usd,
   csv_utils::{CsvWriter,CsvHeader},
   date_utils::parse_date,
   err_utils::ErrStr,
   tuple_utils::{Partition,fst},
   utils::get_env
};

use libs::{
   collections::assets::{mk_assets,assets_by_price},
   fetchers::{fetch_quotes,fetch_pivots},
   reports::{header,total_line,print_table},
   types::{
      assets::{Asset,mk_asset},
      comps::{Composition,mk_composition},
      measurable::{Measurable,tvl},
      pivots::{Pivot,is_virtual,committed},
      quotes::Quotes,
      util::{Blockchain,Token}
   }
};

fn version() -> String { "2.00".to_string() }
fn app_name() -> String { "virtsz".to_string() }

async fn show_virtual_pivots(protocol: &str, dt: &str, p0: &Token, p1: &Token)
      -> ErrStr<()> {
   let pri = p0.to_uppercase();
   let piv = p1.to_uppercase();
   let auth = protocol.to_uppercase();
   let root_url = get_env(&format!("{auth}_URL"))?;
   let date = parse_date(&dt)?;
   let quotes = fetch_quotes(&date).await?;
   let truz = &quotes.aliases;
   let mut aggs = Vec::new();
   let mut asts = mk_assets();
   let (all_opns, _cls, _mx) = fetch_pivots(&root_url, &pri, &piv, truz).await?;
   let (virts, _opns): Partition<Pivot> =
       all_opns.into_iter().partition(is_virtual);
   virts.iter().for_each(|v| asts.add(committed(&v)));

/* 4 scenarii: 

1. no matches, no virtual pivots
2. 1 match on primary
3. 1 match on pivot
4. 2 matches: primary, pivot

so, you know: handle those.
*/
   let pool_name = header(&pri, &piv);
   if !asts.is_empty() {   // no matches case
      let abp = assets_by_price(&asts);

      if let [pr, pv] = abp.as_slice() {
         let blk = fst(pv.key());
         let comp = mk_composition(pr.clone(), pv.clone());
         aggs.push(comp);
         fn nonce<'a>(b: &'a Blockchain, dt: &'a NaiveDate, q: &'a Quotes)
               -> impl Fn(&'a Token) -> ErrStr<Asset> {
            move |tok| {
               let qt = q.lookup(&tok)?;
               Ok(mk_asset(&(b.clone(), tok.clone()), 0.0, &mk_usd(qt), dt))
            }
         }
         let zed = nonce(&blk, &date, &quotes);
         asts.add(zed(&pri)?);
         asts.add(zed(&piv)?);
      } else {
         panic!("Not two assets in {pool_name} Assets: {:?}", abp)
      }
   } else {
      println!("Pivot pool {pool_name} has no virtual pivots.");
   }
   report_on_assets(&aggs, &virts);
   Ok(())
}

fn report_on_assets(pools: &Vec<Composition>, virts: &Vec<Pivot>) {
   println!("{}, version: {}", app_name(), version());
   tabl("Virtual Pivot Assets", pools);
   tabl("Virtual pivots", virts);
}

fn tabl<T:CsvWriter + CsvHeader + Measurable>(title: &str, rows: &Vec<T>) {
   let skip = if let Some(a_row) = rows.first() { a_row.ncols() } else {
      panic!("Portfolio has no pivot pools!")
   } - 3;
   print_table(title, rows);
   total_line(skip, " ,total", &rows.iter().map(tvl).sum());
}

fn usage() -> String {
   println!("\n$ ./{} <protocol> <date> <prim> <piv>

Computes assets committed to virtual pivots.

where
* <protocol> is the protocol, e.g. PIVOT
* <date> to check availability, e.g.: $LE_DATE
* <prim> primary asset, e.g.: BTC
* <piv> pivot asset, e.g.: ETH
", app_name());
   "Needs arguments <protocol> <date> <prim> <piv>".to_string()
}

pub mod functional_tests {
   use super::*;
   use book::{ date_utils::yesterday, utils::get_args };

   pub async fn runoff_with_args() -> ErrStr<()> {
      let args = get_args();
      if let [protocol, dt, pri, piv] = args.as_slice() {
         show_virtual_pivots(&protocol, &dt, pri, piv).await
      } else { Err(usage()) }
   }

   pub async fn runoff() -> ErrStr<usize> {
      let yday = format!("{}", yesterday());
      fn s(t: &str) -> Token { t.to_string() }
      let _ = show_virtual_pivots("pivot", &yday, &s("btc"), &s("eth")).await?;
      Ok(1)
   }
}


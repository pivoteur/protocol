use chrono::NaiveDate;

use book::{
   currency::usd::mk_usd,
   csv_utils::CsvWriter,
   date_utils::parse_date,
   err_utils::ErrStr,
   tuple_utils::fst,
   utils::{get_args,get_env}
};

use libs::{
   collections::assets::{mk_assets,assets_by_price},
   fetchers::{fetch_quotes,fetch_open_pivots},
   git::fetch_pool_names,
   reports::{header,total_line,print_table,compact},
   types::{
      aliases::{Aliases,aliases},
      assets::{Asset,mk_asset},
      comps::{Composition,mk_composition,total},
      pivots::{is_virtual,committed},
      quotes::{Quotes,lookup},
      util::{Blockchain,Token}
   }
};

fn version() -> String { "1.02".to_string() }
fn app_name() -> String { "virtsz".to_string() }

#[tokio::main]
async fn main() -> ErrStr<()> {
   let args = get_args();
   if args.len() < 2 { Err(usage()) } else { show_me(args).await }
}

async fn show_me(args: Vec<String>) -> ErrStr<()> {
   if let [protocol, dt] = &args.as_slice() {
      let auth = protocol.to_uppercase();
      let root_url = get_env(&format!("{auth}_URL"))?;
      let date = parse_date(&dt)?;
      let quotes = fetch_quotes(&date).await?;
      let mut virts = Vec::new();
      let mut no_virts = Vec::new();
      let pool_names = fetch_pool_names(&auth, "data/pools").await?;
      let truz = aliases();
      for (pri, piv) in pool_names {
         eprintln!("*** trying: {}", header(&pri, &piv));
         let mut asts = mk_assets();
         let mut key = (String::new(), String::new());
         let (open_pivs, _) = fetch_open_pivots(&root_url, &pri, &piv).await?;
         for pivot in open_pivs {
            if is_virtual(&pivot) { 
               let cmt = committed(&pivot);
               key = cmt.key();
               asts.add(cmt);
            }
         }

/* 4 scenarii: 

1. no matches, no virtual pivots
2. 1 match on primary
3. 1 match on pivot
4. 2 matches: primary, pivot

so, you know: handle those.
*/
         if asts.is_empty() {   // no matches case
            no_virts.push((pri, piv));
         } else {
            let blk = fst(key);
            fn nonce<'a>(b: &'a Blockchain, dt: &'a NaiveDate,
                         q: &'a Quotes, a: &'a Aliases)
               -> impl Fn(&'a Token) -> ErrStr<Asset> {
               move |token| {
                  let tok = a.alias(token);
                  let qt = lookup(&q, &tok)?;
                  Ok(mk_asset(&(b.clone(), tok.clone()), 0.0, &mk_usd(qt), dt))
               }
            }
            let zed = nonce(&blk, &date, &quotes, &truz);
            asts.add(zed(&pri)?);
            asts.add(zed(&piv)?);
            let abp = assets_by_price(&asts);

            if let [pr, pv] = abp.as_slice() {
               let comp = mk_composition(pr.clone(), pv.clone());
               virts.push(comp);
            } else {
               panic!("Not two assets in {} Assets: {:?}",
                      header(&pri, &piv), abp)
            }
         }
      }
      report_on_assets(&virts);
      compact("Pivot pools with no virtual pivots", "", &no_virts, 
              virts.first(), 12);
   }
   Ok(())
}

fn report_on_assets(pools: &Vec<Composition>) {
   let skip = if let Some(a_pool) = pools.first() { a_pool.ncols() } else {
      panic!("Portfolio has no pivot pools!")
   } - 3;
   println!("{}, version: {}", app_name(), version());
   print_table("Virtual Pivot Assets", pools);
   total_line(skip, " ,total", &total(pools));
}

fn usage() -> String {
   println!("\n$ ./{} <protocol> <date>

Computes assets committed to virtual pivots.

where
* <protocol> is the protocol, e.g. PIVOT
* <date> to check availability, e.g.: $LE_DATE
", app_name());
   "Needs arguments <protocol> <date>".to_string()
}

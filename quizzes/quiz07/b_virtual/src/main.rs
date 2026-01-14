use book::{
   date_utils::parse_date,
   err_utils::ErrStr,
   num_utils::parse_or,
   utils::{get_args,get_env}
};

use libs::{
   collections::assets::{mk_assets,assets_by_price},
   fetchers::{fetch_quotes,fetch_open_pivots},
   git::fetch_pool_names,
   reports::{header,print_table},
   types::{
      comps::mk_composition,
      pivots::{is_virtual,committed},
      quotes::lookup
   }
};

#[tokio::main]
async fn main() -> ErrStr<()> {
   let args = get_args();
   if args.len() < 2 { Err(usage()) } else { show_me(args).await }
}

async fn show_me(args: Vec<String>) -> ErrStr<()> {
   if let [protocol, dt] = &args {
      let auth = protocol.to_uppercase();
      let root_url = get_env(&format!("{auth}_URL"))?;
      let date = parse_date(&dt)?;
      let quotes = fetch_quotes(&date).await?;
      let mut pools = Vec::new();
      let pool_names = fetch_pool_names(&auth, "data/pools").await?;
      for (pri, piv) in pool_names {
         let mut asts = mk_assets();
         let (open_pivs, _) = fetch_open_pivots(&root_url, &pri, &piv).await?;
         for pivot in open_pivs {
            if is_virtual(&pivot) { asts.add(committed(&pivot)); }
         }
         let abp = assets_by_price(&asts);
/* 4 scenarii: 

1. no matches, no virtual pivots
2. 1 match on primary
3. 1 match on pivot
4. 2 matches: primary, pivot

so, you know: handle those.
*/
         if let (pr, pv) = &abp {
            let comp = mk_composition(&pr, &pv);
            pools.push(comp);
         } else {
      }
      print_table("Virtual Pivot Assets:", &open_pivs);
   }
   Ok(())
}

fn usage() -> String {
   println!("\n$ ./aurora <protocol> <date> [min=1000]

Computes assets committed to virtual pivots.

where
* <protocol> is the protocol, e.g. PIVOT
* <date> to check availability, e.g.: $LE_DATE
");
   "Needs arguments <protocol> <date>".to_string()
}

use book::{
   csv_utils::CsvWriter,
   date_utils::parse_date,
   err_utils::ErrStr,
   utils::{get_args,get_env}
};

use libs::{
   fetchers::fetch_quotes,
   git::fetch_pool_names,
   reports::{total_line,print_table,compact},
   virtuals::virtuals,
   types::{
      aliases::aliases,
      comps::{Composition,total}
   }
};

fn version() -> String { "1.03".to_string() }
fn app_name() -> String { "virtsz".to_string() }

#[tokio::main]
async fn main() -> ErrStr<()> {
   let args = get_args();
   if let [protocol, dt] = args.as_slice() {
      compute_virtuals(&protocol, &dt).await
   } else {
      Err(usage())
   }
}

async fn compute_virtuals(protocol: &str, dt: &str) -> ErrStr<()> {
   let auth = protocol.to_uppercase();
   let root_url = get_env(&format!("{auth}_URL"))?;
   let date = parse_date(&dt)?;
   let quotes = fetch_quotes(&date).await?;
   let mut virts = Vec::new();
   let mut no_virts = Vec::new();
   let a = aliases();
   let pool_names = fetch_pool_names(&auth, "data/pools").await?;
   for pool in pool_names {
      let mb_virts = virtuals(&root_url, &date, &a, &quotes, &pool).await?;
      let _: Option<()> = mb_virts.and_then(|v| { virts.push(v); Some(()) })
                                  .or_else(|| { no_virts.push(pool); None });
   }
   report_on_assets(&virts);
   compact("Pivot pools with no virtual pivots", "",
           &no_virts, virts.first(), 12);
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

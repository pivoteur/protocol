use chrono::NaiveDate;

use book::{
   date_utils::parse_date,
   err_utils::ErrStr,
   num::floats::mk_safe_float,
   utils::{ get_args, get_env }
};

use libs::{
   fetchers::{
      assets::pool::fetch_available_assets,
      pool_names::fetch_pool_names,
      quotes::fetch_quotes
   },
   types::comps::Composition
};

fn version() -> String { "1.00".to_string() }
fn app_name() -> String { "hwaet".to_string() }
fn usage() -> ErrStr<()> {
   let app = app_name();
   println!("\n{}, version {}\n\n\t$ {} <protocol> <date>

prints the current available assets for all pivot pools: a health-check.

where
* <protocol> is the protocol, e.g. PIVOT
* <date> to check availability, e.g.: $LE_DATE
", app,  version(), app);
   Err("Needs arguments <protocol> <date>".to_string())
}

async fn compute_health(auth: &str, date: &NaiveDate)
      -> ErrStr<Vec<Composition>> {
   let aut = auth.to_uppercase();
   let root_url = get_env(&format!("{aut}_URL"))?;
   let pools = fetch_pool_names(&root_url).await?;
   let quotes = fetch_quotes(date).await?;
   let mut ans = Vec::new();
   for pool in pools {
      let comp = fetch_available_assets(auth, &quotes, &pool).await?;
      ans.push(comp);
   }
   ans.sort_by_key(|c| mk_safe_float(&c.tvl().amount));
   Ok(ans)
}

fn composition_as_js_health_row(c: &Composition) -> String {
   format!("{{ pool: {:?}, available: '{}' }}",
           c.pool_name(), c.tvl())
}

fn report_health(dt: NaiveDate, v: Vec<Composition>) {
   let pools: Vec<String> =
      v.iter().map(composition_as_js_health_row).collect();
   println!("const poolHealth = {{");
   println!("   generated: '{dt}',
   pools = [
      {}
   ]
}};", pools.join("\n      "));
}

pub async fn runoff_with_args() -> ErrStr<()> {
   let args = get_args();
   if let [auth, dt] = args.as_slice() {
      let date = parse_date(&dt)?;
      let comps = compute_health(&auth, &date).await?;
      report_health(date, comps);
      Ok(())
   } else {
      usage()
   }
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, date_utils::yesterday, utils::now };

   create_testing!("quiz07::d_health");

   run!("compute_health", {
      let yday = yesterday();
      let comps = now(compute_health("pivot", &yday))?;
      report_health(yday, comps);
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use std::collections::HashSet;
   use book::{ date_utils::yesterday, utils::get_env };
   use libs::types::util::pool_name;

   #[tokio::test] async fn test_compute_health_ok() {
      assert!(compute_health("pivot", &yesterday()).await.is_ok());
   }

   #[tokio::test] async fn test_compute_health_all_pools() -> ErrStr<()> {
      let yday = yesterday();
      let auth = "PIVOT";
      let root_url = get_env(&format!("{auth}_URL"))?;
      let npools = fetch_pool_names(&root_url).await?;
      let pool_names: HashSet<String> = npools.iter().map(pool_name).collect();
      let assets = compute_health(auth, &yday).await?;
      let al = &assets.len();
      let pl = &pool_names.len();
      assert_eq!(pl, al, "Assets {al} do not equal pools {pl}!");
      for a in assets {
         let asset = a.pool_name();
         assert!(pool_names.contains(&asset),
                 "I do not know this pool: {asset}");
      }
      Ok(())
   }
}


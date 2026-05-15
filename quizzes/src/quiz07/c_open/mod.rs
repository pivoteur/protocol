use chrono::NaiveDate;

use book::{
   csv_utils::CsvWriter,
   date_utils::parse_date,
   err_utils::ErrStr,
   utils::{get_env,get_args}
};

use libs::{
   fetchers::{
      quotes::fetch_quotes,
      assets::pool::fetch_assets,
      pivots::fetch_pivots
   },
   types::{
      comps::{Composition,from_assets},
      pivots::{Pivot,pivot_assets},
      quotes::Quotes,
      util::{pool_from_str,Pool,pool_name}
   }
};

fn version() -> String { "2.01".to_string() }
fn app_name() -> String { "aurora".to_string() }
fn usage() -> ErrStr<()> {
   let app = app_name();
   println!("\n{}, version {}\n\n\t$ {} <protocol> <date> <pool>

Creates virtual pivots based upon available assets.

where
* <protocol> is the protocol, e.g. PIVOT
* <date> to check availability, e.g.: $LE_DATE
* <pool> pool to open pools, e.g.: btc-eth
", app,  version(), app);
   Err("Needs arguments <protocol> <date>".to_string())
}

async fn enfetchify(auth: &str, quotes: &Quotes, pool: &Pool)
      -> ErrStr<(Composition, Vec<Pivot>)> {
   let aut = auth.to_uppercase();
   let root_url = get_env(&format!("{aut}_URL"))?;
   let aliases = &quotes.aliases;
   let (prim, piv) = pool;
   let pool_assets = fetch_assets(&root_url, &prim, &piv, aliases).await?;
   let ((opens, _closes), _max_date) =
      fetch_pivots(&root_url, &prim, &piv, aliases).await?;
   Ok((pool_assets, opens))
}

async fn fetch_available_assets(auth: &str, date: NaiveDate, pool: &Pool)
      -> ErrStr<Composition> {
   let quotes = fetch_quotes(&date).await?;
   let (pool_assets, opens) = enfetchify(auth, &quotes, pool).await?;
   let mut available = pool_assets.as_assets();
   let all_opens = pivot_assets(&opens)?;
   for a in all_opens.assets() {
      available.subtract(&a);
   }
   available.update_prices(&quotes)?;
   from_assets(&available.assets())
}

pub async fn runoff_with_args() -> ErrStr<()> {
   let args = get_args();
   if let [auth, dt, pool_str] = args.as_slice() {
      let date = parse_date(&dt)?;
      let pool = pool_from_str(pool_str)?;
      let comp = fetch_available_assets(&auth, date, &pool).await?;
      println!("Available assets for {} pivot pool are:
{}", pool_name(&pool), comp.as_csv());
      Ok(())
   } else {
      usage()
   }
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{
      create_testing,
      date_utils::yesterday,
      utils::now
   };
   use libs::types::util::pool_from_str;

   async fn fetchme() -> ErrStr<Composition> {
      let yday = yesterday();
      let pool = pool_from_str("btc-eth")?;
      fetch_available_assets("pivot", yday, &pool).await
   }

   create_testing!("quiz07::c_open");

   run!("fetch_available_assets", {
      let yday = yesterday();
      let pool = pool_from_str("btc-eth")?;
      let comp = now(fetch_available_assets("pivot", yday, &pool))?;
      println!("\nAvailable BTC+ETH assets\n{}", comp.as_csv());
   });

   #[cfg(test)]
   mod tests {
      use super::*;
      use book::{
         currency::usd::{USD,no_monay},
         utils::{composer,deref}
      };
      use libs::types::measurable::tvl;

      #[tokio::test]
      async fn test_fetch_available_assets_ok() {
         let fetchèð = fetchme().await;
         assert!(fetchèð.is_ok());
      }

      fn tvlr() -> impl Fn(Pivot) -> USD {
         composer(deref(tvl), composer(Result::unwrap, deref(Pivot::committed)))
      }

      async fn enfetchme() -> ErrStr<(Composition, Vec<Pivot>)> {
         let yday = yesterday();
         let pool = pool_from_str("btc-eth")?;
         let quotes = fetch_quotes(&yday).await?;
         enfetchify("pivot", &quotes, &pool).await
      }

      #[tokio::test]
      async fn test_fetch_available_assets() -> ErrStr<()> {
         let (comp, _pivs) = enfetchme().await?;
         assert!(comp.tvl() > no_monay(), "BTC+ETH Pool composition zero!");
         Ok(())
      }
      #[tokio::test]
      async fn test_enfetchify_has_pivots() -> ErrStr<()> {
         let (_comp, pivs) = enfetchme().await?;
         assert!(!pivs.is_empty(), "No open BTC+ETH pivots?");
         Ok(())
      }
      #[tokio::test]
      async fn test_enfetchify_pivots() -> ErrStr<()> {
         let (_comp, pivs) = enfetchme().await?;
         let pivs_tvls: USD = pivs.into_iter().map(tvlr()).sum();
         assert!(pivs_tvls > no_monay(), "BTC+ETH pivots amount to $0.00?");
         Ok(())
      }

      #[tokio::test]
      async fn test_enfetchify_assets_and_pivots() -> ErrStr<()> {
         let comp = fetchme().await?;
         assert!(comp.tvl() > no_monay(),
                 "BTC+ETH pivot pool:
{}
has negative balance: {}", comp.as_csv(), comp.tvl());
         Ok(())
      }
   }
}


use book::{
   date_utils::parse_date,
   err_utils::ErrStr,
   num_utils::parse_or,
   utils::get_env
};

use libs::{
   fetchers::{fetch_quotes,fetch_assets,fetch_open_pivots},
   git::fetch_pool_names,
   reports::{header,print_table}
};

async fn list_quotes_and_assets(args: Vec<String>) -> ErrStr<()> {
   let mb_min = if args.len() > 2 { args.last() } else { None };
   let _min_pivot_amt = parse_or(mb_min, 1000.0);
   if let [protocol, dt] = &args[0..2] {
      let auth = protocol.to_uppercase();
      let root_url = get_env(&format!("{auth}_URL"))?;
      let date = parse_date(&dt)?;
      let quotes = fetch_quotes(&date).await?;
      let aliases = &quotes.aliases.clone();
      print_table("Quotes:", &[quotes]);
      let pool_names = fetch_pool_names(&auth, "data/pools").await?;
      for (pri, piv) in pool_names {
         let pool = fetch_assets(&root_url, &pri, &piv, aliases).await?;
         print_table(&format!("Pool {}:", header(&pri, &piv)), &[pool]);
         let (open_pivs, _) =
            fetch_open_pivots(&root_url, &pri, &piv, aliases).await?;
         print_table("Open Pivots:", &open_pivs);
      }
   }
   Ok(())
}

fn usage() -> String {
   println!("\n$ ./aurora <protocol> <date> [min=1000]

Computes available assets to pivot.

where
* <protocol> is the protocol, e.g. PIVOT
* <date> to check availability, e.g.: $LE_DATE
* [min] minimum pivot amount (default $1000.00)
");
   "Needs arguments <protocol> <date>, optionally [min=1000]".to_string()
}

// ----- TESTS -------------------------------------------------------

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use book::{date_utils::yesterday, string_utils::words, utils::get_args};

   pub async fn runoff_with_args() -> ErrStr<()> {
      let args = get_args();
      if args.len() < 2 {
         Err(usage())
      } else {
         list_quotes_and_assets(args).await
      }
   }

   pub async fn runoff() -> ErrStr<usize> {
      let yday = yesterday();
      let _ = list_quotes_and_assets(words(&format!("pivot {yday}"))).await?;
      Ok(1)
   }
}


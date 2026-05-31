use book::{
   err_utils::ErrStr,
   string_utils::{str2strf,words},
   types::filters::{ mk_whitelist, WhiteList },
   utils::{composer,get_env}
};
use super::{ utils::fetch_lines };
use crate::paths::data_dir;

pub async fn fetch_whitelist(auth: &str, file: &str)
      -> ErrStr<WhiteList<String>> {
   let aut = auth.to_uppercase();
   let root_url = get_env(&format!("{}_URL", aut))?;
   let file_name = format!("{}/{file}", data_dir(&root_url));
   let lines = fetch_lines(&file_name).await?;

   fn filter_0x(v: Vec<String>) -> Vec<String> {
      v.into_iter().filter(|s| s.starts_with("0x")).collect()
   }
   let addys: Vec<String> =
      lines.into_iter()
           .map(composer(filter_0x,str2strf(words)))
           .flatten()
           .collect();
   Ok(mk_whitelist(addys))
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, csv_utils::CsvWriter, utils::now };

   create_testing!("fetchers::whitelist");
   run!("fetch_whitelist", {
      let wl = now(fetch_whitelist("pivot", "pivot-token-addresses.txt"))?;
      println!("The Pivot Protocol whitelist is
{}", wl.as_csv());
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;

   #[tokio::test] async fn test_fetch_whitelist_ok() {
      let wl = fetch_whitelist("pivot", "pivot-token-addresses.txt").await;
      assert!(wl.is_ok());
   }

   #[tokio::test] async fn fail_fetch_whitelist() {
      let ur_dad = fetch_whitelist("pivot", "ur_mom.txt").await;
      assert!(ur_dad.is_err());
   }
}


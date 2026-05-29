use book::{
   err_utils::ErrStr,
   types::filters::{ mk_whitelist, WhiteList },
   utils::get_env
};
use super::{ utils::fetch_lines };

pub async fn fetch_whitelist(auth: &str, file: &str)
      -> ErrStr<WhiteList<String>> {
   let aut = auth.to_uppercase();
   let root_url = get_env(&format!("{}_URL", aut))?;
   let data_dir = get_env(&format!("{}_DATA_DIR", aut))?;
   let file_name = format!("{root_url}/{data_dir}/{file}");
   let text = fetch_lines(&file_name).await?;
   Ok(mk_whitelist(text))
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, utils::now };

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
   use book::{ csv_utils::CsvWriter, types::filters::PermissionList };

   #[tokio::test] async fn test_fetch_whitelist_ok() {
      let wl = fetch_whitelist("pivot", "pivot-token-addresses.txt").await;
      assert!(wl.is_ok());
   }

   #[tokio::test] async fn fail_fetch_whitelist() {
      let ur_dad = fetch_whitelist("pivot", "ur_mom.txt").await;
      assert!(ur_dad.is_err());
   }
}


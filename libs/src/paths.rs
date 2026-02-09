// ----- location of the pivot-files ----------------------------------------

fn raw_url(root_url: &str) -> String {
   format!("{}/refs/heads/main", root_url)
}

fn open_pivots_url(root_url: &str) -> String {
   format!("{}/data/pivots/open/raw", raw_url(root_url))
}

fn pool_file(primary: &str, pivot: &str) -> String {
   format!("{primary}-{pivot}.tsv")
}

pub fn pool_assets_url(root_url: &str, primary: &str, pivot: &str) -> String {
   format!("{}/data/pools/{}", raw_url(root_url), pool_file(primary, pivot))
}

/// Resolves the pivot-assets to the open pivot pool URL
pub fn open_pivot_path(root_url: &str, primary: &str, pivot: &str) -> String {
   format!("{}/{}", open_pivots_url(root_url), pool_file(primary, pivot))
}

// ----- For to extract the quotes of the day ---------------------------------

fn lg_raw_url() -> String {
   "https://raw.githubusercontent.com/logicalgraphs/crypto-n-rust".to_string()
}

/// URL to pull the table of quotes reposed in git
pub fn quotes_url() -> String {
   format!("{}/refs/heads/main/data-files/csv/quotes.csv", lg_raw_url())
}

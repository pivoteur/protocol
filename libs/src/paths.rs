fn root_url() -> String {
   "https://raw.githubusercontent.com/pivoteur/pivoteur.github.io".to_string()
}

fn raw_url() -> String {
   format!("{}/refs/heads/main", root_url())
}

fn open_pivots_url() -> String {
   format!("{}/data/pivots/open/raw", raw_url())
}
   
/// Resolves the pivot-assets to the open pivot pool URL
pub fn open_pivot_path(primary: &str, pivot: &str) -> String {
   format!("{}/{primary}-{pivot}.tsv", open_pivots_url())
}

fn lg_raw_url() -> String {
   "https://raw.githubusercontent.com/logicalgraphs/crypto-n-rust".to_string()
}

/// URL to pull the table of quotes reposed in git
pub fn quotes_url() -> String {
   format!("{}/refs/heads/main/data-files/csv/pivots.csv", lg_raw_url())
}

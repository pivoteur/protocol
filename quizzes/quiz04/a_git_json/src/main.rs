use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};

use book::{
   err_utils::{ErrStr,err_or},
   utils::{get_env,get_args}
};

#[tokio::main]
async fn main() -> ErrStr<()> {
   if let Some(token) = get_args().first() {
      let header = build_header_with(token)?;
      let json = read_rest_with(header, 
   } else {
      Err("No token to query git repository".to_string())
   }
}

fn mk_url(owner: &str, repo: &str, path: &str) -> String {
   format!("https://api.github.com/repos/{owner}/{repo}/contents/{path}")
}

async fn read_rest_with(hm: HeaderMap, url: &str) -> ErrStr<String> {

}

fn build_header_with(token: &str) -> ErrStr<HeaderMap> {

}

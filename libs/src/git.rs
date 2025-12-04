use reqwest::header::{HeaderMap, HeaderValue,AUTHORIZATION,ACCEPT,USER_AGENT};

use book::{
   err_utils::ErrStr,
   utils::get_env
};

pub fn marshall_git_call(token: &str) -> ErrStr<(HeaderMap, String)> {
   let header = build_header_with(token)?;
   let ownr = owner(token)?;
   let rep = repo(&ownr);
   let path = "data/pivots/open/raw/";
   let url = mk_url(&ownr, &rep, path);

   Ok((header, url))
}

// You can use booK::rest_utils::read_rest_with to make the call.

// ----- private functions -----------------------------------------------

fn mk_url(owner: &str, repo: &str, path: &str) -> String {
   format!("https://api.github.com/repos/{owner}/{repo}/contents/{path}")
}

fn owner(token: &str) -> ErrStr<String> {
   match token {
      "PIVOT" => Ok("pivoteur".to_string()),
      "LG" => Ok("logicalgraphs".to_string()),
      _    => Err(format!("No owner for token {token}"))
   }
}

fn repo(owner: &str) -> String { format!("{owner}.github.io") }

fn build_header_with(token: &str) -> ErrStr<HeaderMap> {
   let mut hm = HeaderMap::new();
   hm.insert(ACCEPT, HeaderValue::from_static("application/vnd.github.object"));
   let tok = get_env(&format!("{token}_GIT_TOKEN"))?;
   hm.insert(AUTHORIZATION,
             HeaderValue::from_str(&format!("BEARER {tok}")).unwrap());
   hm.insert("X-GitHub-Api-Version", HeaderValue::from_static("2022-11-28"));
   hm.insert(USER_AGENT, HeaderValue::from_static("PivotProtocol"));
   Ok(hm)
}


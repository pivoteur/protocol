use reqwest::header::{HeaderMap, HeaderValue,AUTHORIZATION,ACCEPT,USER_AGENT};
use serde::{Deserialize, Serialize};

use book::{
   err_utils::{ErrStr,err_or},
   rest_utils::{read_rest_with},
   utils::get_env
};

use crate::types::util::Pool;

pub async fn fetch_pool_names(auth: &str, path: &str) -> ErrStr<Vec<Pool>> {
   let (hdr, url) = marshall_git_call(auth, path)?;
   let json_str = read_rest_with(hdr, &url).await?;
   let json: Root = err_or(serde_json::from_str(&json_str),
       &format!("Could not parse JSON {json_str}"))?;
   let pools: Vec<Pool> = 
      json.entries
          .iter()
          .filter_map(|entry| assets(&entry.name).ok())
          .collect();
   Ok(pools)
}

// ----- private functions -----------------------------------------------

// ----- JSON parsing ----------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
struct Root {
    entries: Vec<Entry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Entry {
    name: String,
}

fn assets(file: &str) -> ErrStr<Pool> {
   let file_parts: Vec<&str> = file.split('.').collect();
   if let Some(name) = file_parts.first() {
      let name_parts: Vec<&str> = name.split('-').collect();
      if let [princ, piv] = name_parts.as_slice() {
         Ok((princ.to_uppercase(), piv.to_uppercase()))
      } else {
         Err(format!("Could not split assets from {name}"))
      }
   } else {
      Err(format!("File {file} is not a file."))
   }
}

// ----- marshalling the call to git to get pool directory contents ------

fn marshall_git_call(token: &str, path: &str) -> ErrStr<(HeaderMap, String)> {
   let header = build_header_with(token)?;
   let ownr = owner(token)?;
   let rep = repo(&ownr);
   let url = mk_url(&ownr, &rep, path);
   Ok((header, url))
}

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


use reqwest::header::{HeaderMap, HeaderValue,AUTHORIZATION,ACCEPT,USER_AGENT};

use book::{
   err_utils::{ErrStr,err_or},
   utils::get_env
};

async fn fetch_opens(auth: &str) -> ErrStr<()> {
   let json = fetch_opens_json(auth).await?;
   report(auth, &json)
}

async fn fetch_opens_json(auth: &str) -> ErrStr<String> {
   let aut = auth.to_uppercase();
   let (header, url) = hdr_url(&aut)?;
   read_rest_with(header, &url).await
}

fn hdr_url(auth: &str) -> ErrStr<(HeaderMap, String)> {
   let header = build_header_with(auth)?;
   let ownr = owner(auth)?;
   let rep = repo(&ownr);
   let path = "data/pivots/open/raw/";
   let url = mk_url(&ownr, &rep, path);
   Ok((header, url))
}

fn report(auth: &str, json: &str) -> ErrStr<()> {
   println!("I got {json} from {auth}");
   Ok(())
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

async fn read_rest_with(hm: HeaderMap, url: &str) -> ErrStr<String> {
   let client = reqwest::Client::new();
   let response = err_or(client
            .get(url)
            .headers(hm.clone())
            .send()
            .await, 
      &format!("Could not get a response from {url} with headers {hm:?}"))?;

   if response.status().is_success() {
            let body = err_or(response.text().await, "no text in response")?;
            Ok(body)
   } else {
      let status = response.status();
      let error_body = err_or(response.text().await, "no error in text")?;
      Err(format!("Error status: {status}; Error body: {error_body}"))
   }
}

/* We're aiming for this:

Accept: application/vnd.github.object
"Authorization: Bearer <YOUR-TOKEN>
"X-GitHub-Api-Version: 2022-11-28
*/

fn build_header_with(auth: &str) -> ErrStr<HeaderMap> {
   let mut hm = HeaderMap::new();
   hm.insert(ACCEPT, HeaderValue::from_static("application/vnd.github.object"));
   let tok = get_env(&format!("{auth}_GIT_TOKEN"))?;
   hm.insert(AUTHORIZATION,
             HeaderValue::from_str(&format!("BEARER {tok}")).unwrap());
   hm.insert("X-GitHub-Api-Version", HeaderValue::from_static("2022-11-28"));
   hm.insert(USER_AGENT, HeaderValue::from_static("PivotProtocol"));
   Ok(hm)
}

// ------ TESTS -------------------------------------------------------

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use book::{ test_utils::preamble, utils::get_args };

   fn module() -> String { "quiz04::a_git_json".to_string() }

   async fn run_fetch_opens() -> ErrStr<usize> {
      fetch_opens("pivot").await?;
      Ok(1)
   }
   pub async fn runoff() -> ErrStr<usize> {
      preamble(&module());
      let a = run_fetch_opens().await?;
      Ok(a)
   }

   pub async fn runoff_with_args() -> ErrStr<()> {
      let args = get_args();
      let auth =
         args.first().ok_or("Need <auth> token to fetch open pivot info")?;
      fetch_opens(&auth).await
   }
}


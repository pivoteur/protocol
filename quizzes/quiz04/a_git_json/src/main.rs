use reqwest::header::{HeaderMap, HeaderValue,AUTHORIZATION,ACCEPT,USER_AGENT};

use book::{
   err_utils::{ErrStr,err_or},
   utils::{get_env,get_args}
};

#[tokio::main]
async fn main() -> ErrStr<()> {
   if let Some(token) = get_args().first() {
      let header = build_header_with(token)?;
      let ownr = owner(token)?;
      let rep = repo(&ownr);
      let path = "data/pivots/open/raw/";
      let url = mk_url(&ownr, &rep, path);
      let json = read_rest_with(header, &url).await?;
      println!("I got {json} from {token}");
      Ok(())
   } else {
      Err("No token to query git repository".to_string())
   }
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


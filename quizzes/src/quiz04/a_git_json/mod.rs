use reqwest::header::HeaderMap;

use book::err_utils::{ErrStr,err_or};

async fn fetch_opens(auth: &str) -> ErrStr<()> {
   let json = fetch_opens_json(auth).await?;
   report(auth, &json)
}

fn report(auth: &str, json: &str) -> ErrStr<()> {
   println!("I got {json} from {auth}");
   Ok(())
}

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

// we create a module within this module to ensure the only function called
// from the outer module is hdr_url, controlling the argument-value at entry.

mod header_builder {
   use book::{ err_utils::ErrStr, utils::get_env };
   use reqwest::header::{HeaderMap,HeaderValue,AUTHORIZATION,ACCEPT,USER_AGENT};

   pub fn hdr_url(auth: &str) -> ErrStr<(HeaderMap, String)> {
      let aut = auth.to_uppercase();
      let header = build_header_with(&aut)?;
      let ownr = owner(&aut)?;
      let rep = repo(&ownr);
      let path = "data/pivots/open/raw/";
      let url = mk_url(&ownr, &rep, path);
      Ok((header, url))
   }

   fn owner(auth: &str) -> ErrStr<String> {
      match auth {
         "PIVOT" => Ok("pivoteur".to_string()),
         "LG" => Ok("logicalgraphs".to_string()),
         _    => Err(format!("No owner for protocol {auth}"))
      }
   }

   /* We're aiming for this:

      Accept: application/vnd.github.object
      "Authorization: Bearer <YOUR-TOKEN>
      "X-GitHub-Api-Version: 2022-11-28
   */

   fn build_header_with(auth: &str) -> ErrStr<HeaderMap> {
      fn bearer(tok: &str) -> HeaderValue {
         HeaderValue::from_str(&format!("BEARER {tok}")).unwrap()
      }
      let mut hm = HeaderMap::new();
      let git_obj = "application/vnd.github.object";
      hm.insert(ACCEPT, HeaderValue::from_static(git_obj));
      let tok = get_env(&format!("{auth}_GIT_TOKEN"))?;
      hm.insert(AUTHORIZATION, bearer(&tok));
      hm.insert("X-GitHub-Api-Version", HeaderValue::from_static("2022-11-28"));
      hm.insert(USER_AGENT, HeaderValue::from_static("PivotProtocol"));
      Ok(hm)
   }

   fn mk_url(owner: &str, repo: &str, path: &str) -> String {
      format!("https://api.github.com/repos/{owner}/{repo}/contents/{path}")
   }

   fn repo(owner: &str) -> String { format!("{owner}.github.io") }

   #[cfg(not(tarpaulin_include))]
   #[cfg(test)]
   mod tests {
      use super::*;

      #[test]
      fn fail_build_header_with() {
         let ans = build_header_with("asdf");
         assert!(ans.is_err());
      }
      
      #[test]
      fn test_build_header_with_ok() {
         let ans = build_header_with("PIVOT");
         assert!(ans.is_ok());
      }

      #[test]
      fn test_build_header_with() -> ErrStr<()> {
         let ans = build_header_with("PIVOT")?;
         assert_eq!(Some(&HeaderValue::from_static("PivotProtocol")),
                    ans.get(USER_AGENT));
         Ok(())
      }

      #[test] fn fail_owner() { assert!(owner("asdf").is_err()); }
      #[test] fn test_owner_ok() { assert!(owner("PIVOT").is_ok()); }
      #[test] fn test_owner() -> ErrStr<()> {
         let own = owner("PIVOT")?;
         assert_eq!("pivoteur", &own);
         Ok(())
      }
   }
}

use header_builder::hdr_url;

async fn fetch_opens_json(auth: &str) -> ErrStr<String> {
   let (header, url) = hdr_url(&auth)?;
   read_rest_with(header, &url).await
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

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod tests {
   use super::*;

   #[tokio::test]
   async fn fail_fetch_opens() { assert!(fetch_opens("asdf").await.is_err()); }

   #[tokio::test]
   async fn test_fetch_opens_ok() {
      assert!(fetch_opens("PIVOT").await.is_ok());
   }
}


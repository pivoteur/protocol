use serde::{Deserialize, Serialize};

use book::{
   utils::get_args,
   err_utils::{ErrStr,err_or},
   rest_utils::read_rest_with
};

use libs::git::marshall_git_call;

#[tokio::main]
async fn main() -> ErrStr<()> {
   if let Some(token) = get_args().first() {
      let (hdr, url) = marshall_git_call(token)?;
      let json_str = read_rest_with(hdr, &url).await?;
      let json: Root = err_or(serde_json::from_str(&json_str),
            &format!("Could not parse JSON {json_str}"))?;
      println!("Pivot pool files:\n");
      for (ix, entry) in json.entries.iter().enumerate() {
         println!("{ix}. {}", entry.name);
      }
      Ok(())
   } else {
      Err("No token to query git repository".to_string())
   }
}

#[derive(Debug, Deserialize, Serialize)]
struct Root {
    entries: Vec<Entry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Entry {
    name: String,
}


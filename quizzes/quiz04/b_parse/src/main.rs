use serde::{Deserialize, Serialize};

use book::{
   utils::get_args,
   err_utils::{ErrStr,err_or},
   rest_utils::read_rest_with
};

use libs::{
   types::util::Token,
   git::marshall_git_call
};

#[tokio::main]
async fn main() -> ErrStr<()> {
   if let Some(token) = get_args().first() {
      let (hdr, url) = marshall_git_call(token)?;
      let json_str = read_rest_with(hdr, &url).await?;
      let json: Root = err_or(serde_json::from_str(&json_str),
            &format!("Could not parse JSON {json_str}"))?;
      println!("Pivot pool files:\n");
      for (ix, entry) in json.entries.iter().enumerate() {
         let (princ, piv) = assets(&entry.name)?;
         princ.to_uppercase();
         piv.to_uppercase();
         println!("{ix}. {princ}+{piv}");
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

fn assets(file: &str) -> ErrStr<(Token, Token)> {
   if let Some(name) = file.split('.').collect().first() {
      if let [princ, piv] = name.split('-').collect().as_slice() {
         Ok((princ, piv))
      } else {
         Err(format!("Could not split assets from {name}"))
      }
   } else {

// TODO: xxx
   }
}

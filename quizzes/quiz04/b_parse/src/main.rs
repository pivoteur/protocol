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
      println!("Pivot pools:\n");
      for (ix, entry) in json.entries.iter().enumerate() {
         let (princ, piv) = assets(&entry.name)?;
         println!("{}. {princ}+{piv}", ix+1);
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

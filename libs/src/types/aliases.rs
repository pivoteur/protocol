use std::{ borrow::Borrow, collections::HashMap };

use super::util::Token;

use book::{
   csv_utils::CsvWriter,
   list_utils::fst_snd,
   string_utils::{ s, words },
   utils::composer
};

type Alias = HashMap<Token, Token>;

#[derive(Debug, Clone)]
pub struct Aliases { aliaz: Alias }

pub fn aliases() -> Aliases {
   let mut aliaz: HashMap<String, String> = 
     words("WBTC BTC IBTC BTC WETH ETH IETH ETH IUSD USDC ISOL SOL STABLE USDC")
         .chunks_exact(2)
         .filter_map(composer(Result::ok, fst_snd))
         .collect();
   aliaz.insert(s("LIQUIDITY POOLS"), s("USDC"));
   Aliases { aliaz }
}

impl CsvWriter for Aliases {
   fn ncols(&self) -> usize { self.aliaz.len() }
   fn as_csv(&self) -> String {
      fn commafy<I>(x: I) -> String
            where I: IntoIterator, I::Item: Borrow<String> {
         x.into_iter()
          .map(|s| s.borrow().clone())
          .collect::<Vec<String>>().join(",")
      }
      let froms = commafy(self.aliaz.keys());
      let tos = commafy(self.aliaz.values());
      format!("{froms}\n{tos}")
   }
}

impl Aliases {
   pub fn alias(&self, t: &str) -> Token {
      let cap = t.to_uppercase();
      if let Some(ans) = self.aliaz.get(&cap).or(Some(&cap)) {
         s(ans)
      } else {
         panic!("No alias for {cap}")
      }
   }

   pub fn enum_headers(&self, headers: Vec<String>)
         -> HashMap<String, usize> {
      let mut ix: usize = 0;
      let mut hdrs = HashMap::new();
      fn ikthos(a: &Aliases, hdr: &str) -> String {
         if let [h, t] = words(hdr).as_slice() {
            format!("{} {t}", a.alias(&h))
         } else { 
            a.alias(hdr)
         }
      }
      for hdr in headers {
         hdrs.insert(ikthos(self, &hdr), ix);
         hdrs.insert(hdr, ix);
         ix += 1;
      }
      hdrs
   }
}

// ---- TESTS ---------------------------------------------------------

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::{ create_testing, err_utils::ErrStr };

   create_testing!("types::aliases");
   run_with!("aliases", &aliases(), CsvWriter::as_csv);
   run!("enum_headers", {
      let a = aliases();
      let hdrs = "WBTC STABLE PAXG iSOL USDt";
      let headers = a.enum_headers(words(hdrs));
      println!("The headers for
{hdrs}
are
{headers:?}");
   });
}

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod tests {
   use super::*;

   #[test] fn test_sol_alias() {
      let a = aliases();
      assert_eq!("SOL", &a.alias("iSOL"));
   }
}


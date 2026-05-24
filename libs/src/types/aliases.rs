use std::{ borrow::Borrow, collections::HashMap };

use super::util::Token;

use book::{ csv_utils::CsvWriter, string_utils::words };

type Alias = HashMap<Token, Token>;

#[derive(Debug, Clone)]
pub struct Aliases { aliaz: Alias }

pub fn aliases() -> Aliases {
   let mut aliaz = HashMap::new();
   fn inserter(h: &mut Alias, t1: &str, t2: &str) {
      h.insert(t1.to_string(), t2.to_string());
   }
   inserter(&mut aliaz, "WBTC", "BTC");
   inserter(&mut aliaz, "IBTC", "BTC");
   inserter(&mut aliaz, "WETH", "ETH");
   inserter(&mut aliaz, "IETH", "ETH");
   inserter(&mut aliaz, "IUSD", "USDC");
   inserter(&mut aliaz, "ISOL", "SOL");
   inserter(&mut aliaz, "STABLE", "USDC");
   inserter(&mut aliaz, "LIQUIDITY POOLS", "USDC");
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
         ans.to_string()
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


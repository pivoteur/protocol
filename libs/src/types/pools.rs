use std::{ cmp::Reverse, fmt, hash::Hash, str::FromStr };

use serde::{ Deserialize, Serialize };

use book::{
   date_utils::today,
   err_utils::ErrStr,
   list_utils::fst_snd,
   num::floats::safe_floats::mk_safe_float,
   string_utils::words,
   tuple_utils::fst
};

use super::{ quotes::mk_quotes, util::Token };

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Hash)]
pub struct Pool { primary: Token, pivot: Token }

impl Eq for Pool { }

impl fmt::Display for Pool {
   fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      write!(formatter, "{}", self.pool_name())
   }
}
impl FromStr for Pool {
   type Err = String;
   fn from_str(elt: &str) -> ErrStr<Self> {
      pool_from_str(elt)
   }
}
pub fn mk_pool(a: &str, b: &str) -> Pool {
   Pool { primary: a.to_uppercase(), pivot: b.to_uppercase() }
}

pub fn construct_pool(quotes: [(Token, f32);2], debug: bool) -> ErrStr<Pool> {
   let mut v: Vec<(&str, f32)> =
      quotes.iter().map(|(k,v)| (k.as_str(), *v)).collect();
   v.push(("USDC", -1.0));
   let dict = mk_quotes(&today(), &v);
   let mut assets: Vec<_> =
      quotes.iter()
            .filter_map(|(t,_)| dict.lookup(&t).ok().and_then(|q| Some((t, q))))
            .collect();
   assets.sort_by_key(|(_, q)| Reverse(mk_safe_float(q)));
   if debug { println!("construct_pool(), sorted assets: {assets:?}"); }
   let (a, b) = fst_snd(&assets.into_iter().map(fst).collect::<Vec<_>>())?;
   Ok(mk_pool(&a, &b))
}
   
impl Pool {
   pub fn pool_name(&self) -> String {
      let Pool {primary,pivot} = self;
      format!("{}+{}", primary.to_uppercase(), pivot.to_uppercase())
   }
   pub fn as_tuple(&self) -> (String, String) {
      let Pool { primary, pivot } = self;
      (primary.to_uppercase(), pivot.to_uppercase())
   }
   pub fn file_name(&self) -> String {
      let Pool { primary, pivot } = self;
      format!("{}-{}", primary.to_lowercase(), pivot.to_lowercase())
   }

   pub fn as_vec(&self) -> Vec<String> {
      let Pool { primary, pivot } = self;
      words(&format!("{primary} {pivot}"))
   }
}

pub fn pool_from_str(pool: &str) -> ErrStr<Pool> {
   let tokens: Vec<&str> = pool.split(['-','+']).collect();
   let [a, b] = match tokens.as_slice() {
      [x, y] => Ok([x, y]),
      _ => Err(format!("Malformed pool name: {pool}"))
   }?;
   Ok(mk_pool(&a, &b))
}

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod functional_tests {
   use super::*;
   use paste::paste;
   use book::create_testing; 

   create_testing!("types::pools");
   run!("pool_functions", "btc-eth", {
      let be = "btc-eth";
      let pool = pool_from_str(be)?;
      println!("\tpool_from_str: {pool}");
      println!("\tpool_name: {}", pool.pool_name());
      println!("\tas_tuple: {:?}", pool.as_tuple());
      println!("\tfile_name: {}", pool.file_name());
   });
}
      
#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use book::string_utils::s;

   #[test] fn test_mk_pool() {
      assert_eq!("BTC+ETH", &mk_pool("btc","eth").to_string());
   }

   #[test] fn test_pool_name() {
      assert_eq!("BTC+USDC", &mk_pool("btc", "usdc").pool_name());
   }

   #[test] fn fail_pool_from_nonpool_str() {
      let ans = pool_from_str("asdfadsf");
      assert!(ans.is_err());
   }

   #[test] fn fail_pool_from_too_many_tokens() {
      let ans = pool_from_str("a-b-c");
      assert!(ans.is_err());
   }

   #[test] fn test_pool_from_str_ok() {
      let ans = pool_from_str("eth-undead");
      assert!(ans.is_ok());
   }

   #[test] fn test_pool_from_str() -> ErrStr<()> {
      let ans = pool_from_str("btc-avax")?;
      assert_eq!("BTC+AVAX", &ans.to_string());
      Ok(())
   }

   #[test] fn test_as_tuple() -> ErrStr<()> {
      let ans = pool_from_str("undead+usdc")?;
      assert_eq!((s("UNDEAD"), s("USDC")), ans.as_tuple());
      Ok(())
   }

   #[test] fn test_construct_pool_btc_eth() -> ErrStr<()> {
      let p1 = construct_pool([(s("ETH"), 1748.2), (s("BTC"), 62143.1)], true)?;
      let p2 = construct_pool([(s("btc"), 62443.1), (s("eth"), 1717.1)], true)?;
      assert_eq!("BTC+ETH", &format!("{p1}"), "p1 is wrong");
      assert_eq!("BTC+ETH", &format!("{p2}"), "p2 is wrong");
      Ok(())
   }

   #[test] fn test_construct_usdc_pools() -> ErrStr<()> {
      let p1 = construct_pool([(s("AVAX"), 6.28), (s("USDC"), 1.0)], true)?;
      let p2 = construct_pool([(s("usdc"), 1.0), (s("undead"), 0.009)], true)?;
      assert_eq!("AVAX+USDC", &format!("{p1}"), "p1 is wrong");
      assert_eq!("UNDEAD+USDC", &format!("{p2}"), "p2 is wrong");
      Ok(())
   }
}


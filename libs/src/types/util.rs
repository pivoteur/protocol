use book::{ currency::usd::USD, err_utils::ErrStr };

// ----- Your basic types used across all domains -------------------------

pub type Id = usize;
pub type Token = String;
pub type Address = String;

pub type Pool = (Token, Token);
pub fn mk_pool(a: &str, b: &str) -> Pool {
   (a.to_uppercase(), b.to_uppercase())
}
pub fn pool_name(p: &Pool) -> String {
   let (a,b) = p;
   format!("{}+{}", a.to_uppercase(), b.to_uppercase())
}
pub fn pool_from_str(pool: &str) -> ErrStr<Pool> {
   let tokens: Vec<&str> = pool.split("-").collect();
   let [a, b] = match tokens.as_slice() {
      [x, y] => Ok([x, y]),
      _ => Err(format!("Malformed pool name: {pool}"))
   }?;
   Ok(mk_pool(&a, &b))
}

pub type Blockchain = String; // enum? maybe, but String for now.

pub type TVLs = Vec<(Token,USD)>;

// ----- TESTS -------------------------------------------------------

#[cfg(test)]
mod tests {
   use super::*;
   use book::string_utils::s;

   #[test] fn test_mk_pool() {
      assert_eq!((s("BTC"), s("ETH")), mk_pool("btc","eth"));
   }

   #[test] fn test_pool_name() {
      assert_eq!(s("BTC+USDC"), pool_name(&mk_pool("btc", "usdc")));
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
      assert_eq!((s("BTC"), s("AVAX")), ans);
      Ok(())
   }
}


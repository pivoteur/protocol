use book::currency::usd::USD;

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

pub type TVLs = Vec<(Token,USD)>;


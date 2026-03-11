// ----- Your basic types used across all domains -------------------------

pub type Id = usize;
pub type Token = String;
pub type Pool = (Token, Token);

pub fn mk_pool(a: &str, b: &str) -> Pool { (a.to_string(), b.to_string()) }

pub type Blockchain = String; // enum? maybe, but String for now.


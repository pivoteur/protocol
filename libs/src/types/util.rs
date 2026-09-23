use book::currency::usd::USD;

// ----- Your basic types used across all domains -------------------------

pub type Token = String;
pub type Id = usize;
pub type TVLs = Vec<(Token, USD)>;


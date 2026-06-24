use book::currency::usd::USD;

// ----- Your basic types used across all domains -------------------------

pub type Id = usize;
pub type Token = String;

pub type Blockchain = String; // enum? maybe, but String for now.

pub type TVLs = Vec<(Token,USD)>;


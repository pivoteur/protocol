// ----- Your basic types used across all domains -------------------------

pub type Id = usize;
pub type Token = String;
pub type Pool = (Token, Token);

pub type Blockchain = String; // enum? maybe, but String for now.

// ----- PARTITION type ---------------------------------------------------

pub type Partition<T> = (Vec<T>, Vec<T>);


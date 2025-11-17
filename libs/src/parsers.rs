use std::{
   collections::HashMap,
   hash::Hash
};

use book::{
   err_utils::{ErrStr,err_or},
   tuple_utils::swap
};

use crate::types::util::Id;

pub fn parse_id(s: &str) -> ErrStr<Id> {
   err_or(s.parse(), &format!("{s} is not an Id-type"))
}

pub fn parse_str(s: &str) -> ErrStr<String> {
   Ok(s.to_string())
}

pub fn enum_headers<HEADER: Eq + Hash>(headers: Vec<HEADER>)
      -> HashMap<HEADER, Id> {
   headers.into_iter().enumerate().map(swap).collect()
}


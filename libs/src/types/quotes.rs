use std::collections::HashMap;

use chrono::NaiveDate;

use book::err_utils::ErrStr;

use super::{
   aliases::{Aliases,aliases},
   util::Token
};

#[derive(Clone, Debug)]
pub struct Quotes {
   pub date: NaiveDate,
   quotes: HashMap<Token, f32>,
   aliases: Aliases
}

pub fn mk_quotes(date: NaiveDate, quotes: HashMap<Token, f32>) -> Quotes {
   Quotes { date, quotes, aliases: aliases() }
}

pub fn lookup(q: &Quotes, key: &Token) -> ErrStr<f32> {
   q.quotes.get(&q.aliases.alias(key))
           .ok_or(format!("Unable to find quote for {key}"))
           .copied()
}


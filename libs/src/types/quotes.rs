use std::collections::HashMap;

use chrono::NaiveDate;

use book::err_utils::ErrStr;

use crate::types::util::Token;

#[derive(Clone, Debug)]
pub struct Quotes {
   pub date: NaiveDate,
   quotes: HashMap<Token, f32> 
}

pub fn mk_quotes(date: NaiveDate, quotes: HashMap<Token, f32>) -> Quotes {
   Quotes { date, quotes }
}

pub fn lookup(q: &Quotes, key: &Token) -> ErrStr<f32> {
   q.quotes.get(key).ok_or(format!("Unable to find quote for {key}")).copied()
}

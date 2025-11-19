use std::collections::HashMap;

use chrono::NaiveDate;

pub type Token = String;

#[derive(Clone, Debug)]
pub struct Quotes {
   date: NaiveDate,
   quotes: HashMap<Token, f32> 
}

pub fn mk_quotes(date: NaiveDate, quotes: HashMap<Token, f32>) -> Quotes {
   Quotes { date, quotes }
}

pub fn lookup(q: &Quotes, key: &Token) -> ErrStr<f32> {
   q.quotes.get(key).ok_or(format!("Unable to find quote for {key}"))
}

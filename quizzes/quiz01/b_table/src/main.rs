use chrono::NaiveDate;

use book::{
   date_utils::parse_date,
   err_utils::ErrStr
};

type Id = i32;

#[derive(Debug, Clone)]
struct Header {
   opened: NaiveDate,
   id: Id,
   close: Id
}

fn mk_hdr(opend: &str, id: Id, close: Id) -> ErrStr<Header> {
   let opened = parse_date(opend)?;
   Ok(Header { opened, id, close })
}

#[derive(Debug, Clone)]
struct Amount {
   actual: f32,
   ersatz: f32      // 'ersatz' meaning 'virtual' as 'virtual' is reserved
}

fn amount(a: Amount) -> f32 { a.actual + a.ersatz }
fn mk_amt(actual: f32, ersatz: f32) -> Amount {
   Amount { actual, ersatz }
}

#[derive(Debug, Clone)]
struct Asset {
   token: String,
   amount: Amount
}

fn mk_asset(tkn: &str, amount: Amount) -> Asset {
   Asset { token: tkn.to_string(), amount }
}

#[derive(Debug, Clone)]
struct Pivot {
   header: Header,
   from: Asset,
   to: Asset
}

fn main() -> ErrStr<()> {
   let header = mk_hdr("2025-11-10", 1, 0)?;
   let from = mk_asset("BTC", mk_amt(0.004498, 0.0));
   let to = mk_asset("ETH", mk_amt(0.14203, 0.0));
   let pivot = Pivot { header, from, to };
   println!("Voilà: {pivot:?}");
   Ok(())
}

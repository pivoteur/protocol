use chrono::NaiveDate;

use book::{
   currency::usd::mk_usd,
   err_utils::ErrStr,
   tuple_utils::fst,
   utils::pred
};

use super::{
   collections::assets::{Assets,mk_assets,assets_by_price},
   fetchers::fetch_open_pivots,
   reports::header,
   types::{
      assets::{Asset,mk_asset},
      comps::{Composition,mk_composition},
      pivots::{is_virtual,committed},
      quotes::Quotes,
      util::{Blockchain,Token,Pool}
   }
};

type Key = (Blockchain, Token);
fn null_key() -> Key { (String::new(), String::new()) }

pub async fn virtuals(root_url: &str, dt: &NaiveDate, q: &Quotes, pool: &Pool)
      -> ErrStr<Option<Composition>> {
   let (pri, piv) = pool;
   let mut asts = mk_assets();
   let mut key = null_key();
   let (open_pivs, _dt) = fetch_open_pivots(&root_url, &pri, &piv).await?;
   for pivot in open_pivs {
      if is_virtual(&pivot) {
         let cmt = committed(&pivot);
         key = cmt.key();
         asts.add(cmt);
      }
   }
   Ok(pred(!asts.is_empty(), mk_virtuals(fst(key), pool, q, dt, &mut asts)))
}

fn nonce<'a>(b: &'a Blockchain, q: &'a Quotes, dt: &NaiveDate)
      -> impl Fn(&'a Token) -> ErrStr<Asset> {
   move |token| {
      let qt = q.lookup(&token)?;
      Ok(mk_asset(&(b.clone(), token.clone()), 0.0, &mk_usd(qt), dt))
   }
}

fn mk_virtuals(blk: Blockchain, pool: &Pool, q: &Quotes, 
               dt: &NaiveDate, asts: &mut Assets) -> Composition {
   let (pri, piv) = pool;
   let zed = nonce(&blk, q, dt);
   asts.add(zed(&pri).expect("No quote for {pri}"));
   asts.add(zed(&piv).expect("No quote for {piv}"));
   let abp = assets_by_price(&asts);

   if let [pr, pv] = abp.as_slice() {
      mk_composition(pr.clone(), pv.clone())
   } else {
      panic!("Not two assets in {} Assets: {:?}", header(&pri, &piv), abp)
   }
}

#[cfg(test)]
mod tests {
   use chrono::Local;

   use super::*;

   use std::collections::HashMap;

   use crate::types::quotes::mk_quotes;

   fn today() -> NaiveDate { Local::now().naive_local().date() }

   fn zed(t: &str) -> ErrStr<Asset> {
      let today_local = today();
      let qs: HashMap<Token, f32> =
         [("AVAX".to_string(), 12.15),("USDC".to_string(), 1.0)]
            .iter().cloned().collect();
      let q = mk_quotes(today_local, qs);
      let t1 = t.to_string();
      nonce(&"Avalanche".to_string(), &q, &today_local)(&t1)
   }

   #[test]
   fn test_nonce() -> ErrStr<()> {
      let avax = zed("AVAX")?;
      assert_eq!(mk_usd(12.15), avax.quote);
      assert_eq!(0.0, avax.amount);
      Ok(())
   }

   #[test]
   fn fail_nonce() {
      let bnb = zed("BNB");
      assert!(bnb.is_err());
   }
}

use crate::types::{
   pivots::{ Pivot, mk_pivot },
   quotes::Quotes
};

use book::{
   csv_utils::{ CsvHeader, CsvWriter },
   err_utils::ErrStr,
   string_utils::s
};

// ----- RECOMPUTING VIRTUAL PIVOTS --------------------------------------

pub fn recompute_pivot(quotes: &Quotes, debug: bool)
      -> impl Fn(Pivot) -> ErrStr<Pivot> {
   move |p| {
      if !p.is_virtual() { Err(s("Can only recompute virtual pivots"))
      } else {
         if p.closed() { Err(s("Pivot closed; cannot recompute"))
         } else { recompute1(quotes, p, debug)
         }
      }
   }
}

fn recompute1(quotes: &Quotes, p: Pivot, debug: bool) -> ErrStr<Pivot> {
   if debug { println!("For pivot:\n{}\n{}", p.header(), p.as_csv()); }
   let mb_new_assets = p.recompute_assets(quotes)?;
   Ok(match mb_new_assets {
      Some((from, to)) => {
         let today = &quotes.date;
         let header = p.update_header(today);
         let new_piv1 = mk_pivot(header, from, to);
         if debug { println!("\tRecomputed to:\n{}", new_piv1.as_csv()); }
         new_piv1
      },
      None => {
         if debug { println!("\tNo change"); }
         p
      }
   })
}

// ----- TESTS -----------------------------------------------------------

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
   use super::*;
   use paste::paste;
   use book::create_testing;

   use crate::{
      types::{
         assets::amounts::mk_amt,
         pivots::sample_pivots::mk_btc_usdc_piv,
         quotes::sample_data::sample_quotes_maker
      }
   };

   create_testing!("processors::virtuals");

   run!("recompute_pivot", {
      let piv = mk_btc_usdc_piv(78408.88,mk_amt(0.0,0.1),0,"virtual pivot")?;
      let quotes = sample_quotes_maker(&[("BTC", 80000.0)]);
      let _new_piv = recompute_pivot(&quotes, true)(piv)?;
   });
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
   use super::*;
   use crate::types::{
      assets::amounts::mk_amt,
      pivots::sample_pivots::mk_btc_usdc_piv,
      quotes::sample_data::sample_quotes_maker
   };

   #[test] fn fail_recompute_non_virtual_amt_pivot() -> ErrStr<()> {
      let piv = mk_btc_usdc_piv(78408.88, mk_amt(500.0, 0.0), 0, "https://yo")?;
      let reckt =
         recompute_pivot(&sample_quotes_maker(&[("BTC", 80000.0)]), false)(piv);
      assert!(reckt.is_err());
      if let Err(x) = reckt {
         assert!(x.contains("virtual"));
         Ok(())
      } else { 
         Err(format!("reckt ({reckt:?}) succeeds (???) unfortunately."))
      }
   }

   #[test] fn fail_recompute_non_virtual_tx_pivot() -> ErrStr<()> {
      let piv = mk_btc_usdc_piv(78408.88, mk_amt(0.0, 500.0), 0, "https://yo")?;
      let reckt =
         recompute_pivot(&sample_quotes_maker(&[("BTC", 80000.0)]), false)(piv);
      assert!(reckt.is_err());
      if let Err(x) = reckt {
         assert!(x.contains("virtual"));
         Ok(())
      } else {
         Err(format!("reckt ({reckt:?}) succeeds (???) unfortunately."))
      }
   }  

   #[test] fn fail_recompute_closed_pivot() -> ErrStr<()> {
      let piv = mk_btc_usdc_piv(78408.88,mk_amt(0.0,500.0),1,"virtual pivot")?;
      let reckt =
         recompute_pivot(&sample_quotes_maker(&[("BTC",80000.0)]), false)(piv);
      assert!(reckt.is_err());
      if let Err(x) = reckt {
         assert!(x.contains("close"));
         Ok(())
      } else {
         let cls = "closed pivot recompute";
         Err(format!("{cls} {reckt:?} succeeds (???) unfortunately."))
      }
   }

   #[test] fn test_no_recompute_virtual_pivot_ok() -> ErrStr<()> {
      let piv = mk_btc_usdc_piv(78408.88,mk_amt(0.0, 0.1),0,"virtual_pivot")?;
      assert!(!piv.is_updated());
      let neiner =
         recompute_pivot(&sample_quotes_maker(&[("BTC",65000.0)]), false)(piv);
      assert!(neiner.is_ok());
      assert!(!neiner.unwrap().is_updated());
      Ok(())
   }
}


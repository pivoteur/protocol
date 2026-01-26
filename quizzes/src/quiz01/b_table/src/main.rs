use book::err_utils::ErrStr;

use quiz01::b_table::{ingest_table,print_actives,actives_closeds,amounts};

#[tokio::main]
async fn main() -> ErrStr<()> {
   // println!("Voilà: {:?}", sample_pivot());

// let's read in real open pivot data and first, put those data into a 
// (untyped) table

   let table = ingest_table().await?;

// We have our (unstructured) pivots tablized, now let's reify those pivots
// (... starting with just the opened-date data ... and now adding the rest of
// the header).

// Let's also parse the FROM- and TO-assets.

   let (acts, pass) = actives_closeds(&table)?;

   let a = acts.len();
   let p = pass.len();
   println!("\nThere are {a} active pivots and {p} closed pivots.\n");
   print_actives(&acts);

   let (bag, btc, eth) = amounts(&acts);
   for (k,v) in bag.counts {
      let amt: f32 = if k == "BTC" { btc } else { eth };
      println!("There are {v} {k} open pivots, totaling {amt} {k}");
   }

   Ok(())
}


use book::{
   csv_utils::CsvWriter,
   err_utils::ErrStr,
   utils::get_args
};

use libs::{
   reports::{total_line,print_table,compact},
   virtuals::compute_virtuals,
   types::comps::{Composition,total}
};

fn version() -> String { "1.04".to_string() }
fn app_name() -> String { "virtsz".to_string() }

#[tokio::main]
async fn main() -> ErrStr<()> {
   let args = get_args();
   if let [protocol, dt] = args.as_slice() {
      let (virts, no_virts) = compute_virtuals(&protocol, &dt).await?;
      report_on_assets(&virts); report_on_assets(&virts);
      compact("Pivot pools with no virtual pivots", "",
              &no_virts, virts.first(), 12);
      Ok(())
   } else {
      Err(usage())
   }
}

fn report_on_assets(pools: &Vec<Composition>) {
   let skip = if let Some(a_pool) = pools.first() { a_pool.ncols() } else {
      panic!("Portfolio has no pivot pools!")
   } - 3;
   println!("{}, version: {}", app_name(), version());
   print_table("Virtual Pivot Assets", pools);
   total_line(skip, " ,total", &total(pools));
}

fn usage() -> String {
   println!("\n$ ./{} <protocol> <date>

Computes assets committed to virtual pivots.

where
* <protocol> is the protocol, e.g. PIVOT
* <date> to check availability, e.g.: $LE_DATE
", app_name());
   "Needs arguments <protocol> <date>".to_string()
}

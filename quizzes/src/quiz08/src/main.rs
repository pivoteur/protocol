use super::functional_tests::runoff;

use book::err_utils::ErrStr;

fn main() -> ErrStr<()> {
   let _ = runoff()?;
   Ok(())
}


use book::err_utils::ErrStr;
use book::parse_utils::parse_id;
use book::utils::get_env;
use libs::fetchers::fetch_calls;
use quizzes::quiz08::b_urie::{ header, parse_row };

fn main() -> ErrStr<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Error: not enough arguments.");
        eprintln!("Usage: `panic` <ix> <tx_id> <new_to_actual>");
        eprintln!("Example: `panic` 5 \"asdf\" \"1250.75\"");
        std::process::exit(1);
    }
    let ix_str        = &args[1];
    let tx_id         = &args[2];
    let new_to_actual = &args[3];
    let ix            = parse_id(ix_str)?;
    let root_url      = get_env("PIVOT_URL")?;
    let rt            = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    match rt.block_on(fetch_calls(&root_url)) {
        Ok(t) => {
            println!("{}", header());
            println!("{}", parse_row(&t, ix, tx_id, new_to_actual)?);
        }
        Err(e) => eprintln!("Error: {e}"),
    }
    Ok(())
}

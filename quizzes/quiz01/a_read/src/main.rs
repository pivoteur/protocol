use book::{ err_utils::ErrStr, rest_utils::read_rest };

#[tokio::main]
async fn main() -> ErrStr<()> {
    let pivots = "https://raw.githubusercontent.com/pivoteur/pivoteur.github.io/refs/heads/main/data/pivots/open/raw/btc-eth.tsv";
    let ans = read_rest(pivots).await?;

    println!("I got {ans}");
    Ok(())
}

use book::err_utils::ErrStr;

#[tokio::main]
async fn main() -> ErrStr<()> {
    functional_tests::runoff_with_args().await
}

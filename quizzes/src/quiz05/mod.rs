pub mod a_assets;
pub mod b_dusk_min;

#[cfg(not(tarpaulin_include))]
pub mod functional_tests {
    use super::a_assets::functional_tests::runoff as a;
    use super::b_dusk_min::functional_tests::runoff as b;
    use book::err_utils::ErrStr;

    pub async fn runoff() -> ErrStr<usize> {
        let n1 = a().await?;
        let n2 = b().await?;
        Ok(n1 + n2)
    }
}

#[cfg(test)]
mod finite_test {
    use kzg_bench::tests::finite::*;
    use rust_kzg_mcl::data_types::fr::Fr;
    use rust_kzg_mcl::mcl_methods::init;
    use rust_kzg_mcl::CurveType;

    #[test]
    fn sum_of_two_zeros_is_zero_() {
        assert!(init(CurveType::BLS12_381));
        sum_of_two_zeros_is_zero::<Fr>()
    }
}

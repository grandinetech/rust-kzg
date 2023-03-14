#[cfg(test)]
mod finite_test {
    use kzg_bench::tests::finite::*;
    use mcl_rust::data_types::fr::Fr;
    use mcl_rust::mcl_methods::init;
    use mcl_rust::CurveType;

    #[test]
    fn sum_of_two_zeros_is_zero_() {
        assert!(init(CurveType::BLS12_381));
        sum_of_two_zeros_is_zero::<Fr>()
    }
}

#[cfg(test)]
mod recover_tests {
    use kzg_bench::tests::recover::*;
    use mcl_rust::data_types::fr::Fr;
    use mcl_rust::fk20_fft::FFTSettings;
    use mcl_rust::kzg10::Polynomial;
    use mcl_rust::mcl_methods::init;
    use mcl_rust::CurveType;

    #[test]
    fn recover_simple_() {
        assert!(init(CurveType::BLS12_381));
        recover_simple::<Fr, FFTSettings, Polynomial, Polynomial>();
    }

    #[test]
    fn recover_random_() {
        assert!(init(CurveType::BLS12_381));
        recover_random::<Fr, FFTSettings, Polynomial, Polynomial>();
    }

    #[test]
    fn more_than_half_missing_() {
        assert!(init(CurveType::BLS12_381));
        more_than_half_missing::<Fr, FFTSettings, Polynomial, Polynomial>();
    }
}

#[cfg(test)]
pub mod zero_poly_tests {
    use kzg_bench::tests::zero_poly::*;
    use mcl_rust::data_types::fr::Fr;
    use mcl_rust::fk20_fft::FFTSettings;
    use mcl_rust::kzg10::Polynomial;
    use mcl_rust::mcl_methods::init;
    use mcl_rust::CurveType;

    #[test]
    fn test_reduce_partials_() {
        assert!(init(CurveType::BLS12_381));
        test_reduce_partials::<Fr, FFTSettings, Polynomial>();
    }

    #[test]
    fn reduce_partials_random_() {
        assert!(init(CurveType::BLS12_381));
        reduce_partials_random::<Fr, FFTSettings, Polynomial>();
    }

    #[test]
    fn check_test_data_() {
        assert!(init(CurveType::BLS12_381));
        check_test_data::<Fr, FFTSettings, Polynomial>();
    }

    #[test]
    fn zero_poly_known_() {
        assert!(init(CurveType::BLS12_381));
        zero_poly_known::<Fr, FFTSettings, Polynomial>();
    }

    #[test]
    fn zero_poly_random_() {
        assert!(init(CurveType::BLS12_381));
        zero_poly_random::<Fr, FFTSettings, Polynomial>();
    }

    #[test]
    fn zero_poly_all_but_one_() {
        assert!(init(CurveType::BLS12_381));
        zero_poly_all_but_one::<Fr, FFTSettings, Polynomial>();
    }

    #[test]
    fn zero_poly_252_() {
        assert!(init(CurveType::BLS12_381));
        zero_poly_252::<Fr, FFTSettings, Polynomial>();
    }
}
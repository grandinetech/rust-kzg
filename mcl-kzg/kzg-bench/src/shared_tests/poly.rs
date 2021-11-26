#[cfg(test)]
pub mod poly_tests {
    use kzg_bench::tests::poly::*;
    use mcl_rust::data_types::fr::Fr;
    use mcl_rust::kzg10::Polynomial;
    use mcl_rust::mcl_methods::init;
    use mcl_rust::CurveType;
    use mcl_rust::fk20_fft::FFTSettings;

    #[test]
    fn create_poly_of_length_ten_() {
        assert!(init(CurveType::BLS12_381));
        create_poly_of_length_ten::<Fr, Polynomial>();
    }

    #[test]
    fn poly_eval_check_() {
        assert!(init(CurveType::BLS12_381));
        poly_eval_check::<Fr, Polynomial>();
    }

    #[test]
    fn poly_eval_0_check_() {
        assert!(init(CurveType::BLS12_381));
        poly_eval_0_check::<Fr, Polynomial>();
    }

    #[test]
    fn poly_eval_nil_check_() {
        assert!(init(CurveType::BLS12_381));
        poly_eval_nil_check::<Fr, Polynomial>();
    }

    #[test]
    fn poly_inverse_simple_0_() {
        assert!(init(CurveType::BLS12_381));
        poly_inverse_simple_0::<Fr, Polynomial>();
    }

    #[test]
    fn poly_inverse_simple_1_() {
        assert!(init(CurveType::BLS12_381));
        poly_inverse_simple_1::<Fr, Polynomial>();
    }

    #[test]
    fn poly_mul_direct_test_() {
        assert!(init(CurveType::BLS12_381));
        poly_mul_direct_test::<Fr, Polynomial>();
    }

    #[test]
    fn poly_test_div_() {
        assert!(init(CurveType::BLS12_381));
        poly_test_div::<Fr, Polynomial>();
    }

    #[test]
    fn poly_div_by_zero_() {
        assert!(init(CurveType::BLS12_381));
        poly_div_by_zero::<Fr, Polynomial>();
    }

    #[test]
    fn poly_mul_fft_test_() {
        assert!(init(CurveType::BLS12_381));
        poly_mul_fft_test::<Fr, Polynomial, FFTSettings>();
    }

    #[test]
    fn poly_mul_random_() {
        assert!(init(CurveType::BLS12_381));
        poly_mul_random::<Fr, Polynomial, FFTSettings>();
    }

    #[test]
    fn poly_div_random_() {
        assert!(init(CurveType::BLS12_381));
        poly_div_random::<Fr, Polynomial>();
    }

    #[test]
    fn poly_div_long_test_() {
        assert!(init(CurveType::BLS12_381));
        poly_div_long_test::<Fr, Polynomial>();
    }

    #[test]
    fn poly_div_fast_test_() {
        assert!(init(CurveType::BLS12_381));
        poly_div_fast_test::<Fr, Polynomial>();
    }
}

#[cfg(test)]
mod tests {
    use kzg_bench::tests::poly::{
        create_poly_of_length_ten, poly_div_by_zero, poly_div_fast_test, poly_div_long_test,
        poly_div_random, poly_eval_0_check, poly_eval_check, poly_eval_nil_check,
        poly_inverse_simple_0, poly_inverse_simple_1, poly_mul_direct_test, poly_mul_fft_test,
        poly_mul_random, poly_test_div,
    };
    use rust_kzg_arkworks::kzg_proofs::FFTSettings;
    use rust_kzg_arkworks::kzg_types::FsFr;
    use rust_kzg_arkworks::utils::PolyData;

    #[test]
    fn create_poly_of_length_ten_() {
        create_poly_of_length_ten::<FsFr, PolyData>();
    }

    #[test]
    fn poly_eval_check_() {
        poly_eval_check::<FsFr, PolyData>();
    }

    #[test]
    fn poly_eval_0_check_() {
        poly_eval_0_check::<FsFr, PolyData>();
    }

    #[test]
    fn poly_eval_nil_check_() {
        poly_eval_nil_check::<FsFr, PolyData>();
    }

    #[test]
    fn poly_inverse_simple_0_() {
        poly_inverse_simple_0::<FsFr, PolyData>();
    }

    #[test]
    fn poly_inverse_simple_1_() {
        poly_inverse_simple_1::<FsFr, PolyData>();
    }

    #[test]
    fn poly_test_div_() {
        poly_test_div::<FsFr, PolyData>();
    }

    #[test]
    #[should_panic]
    fn poly_div_by_zero_() {
        poly_div_by_zero::<FsFr, PolyData>();
    }

    #[test]
    fn poly_mul_direct_test_() {
        poly_mul_direct_test::<FsFr, PolyData>();
    }

    #[test]
    fn poly_mul_fft_test_() {
        poly_mul_fft_test::<FsFr, PolyData, FFTSettings>();
    }

    #[test]
    fn poly_mul_random_() {
        poly_mul_random::<FsFr, PolyData, FFTSettings>();
    }

    #[test]
    fn poly_div_random_() {
        poly_div_random::<FsFr, PolyData>();
    }

    #[test]
    fn poly_div_long_test_() {
        poly_div_long_test::<FsFr, PolyData>()
    }

    #[test]
    fn poly_div_fast_test_() {
        poly_div_fast_test::<FsFr, PolyData>()
    }
}

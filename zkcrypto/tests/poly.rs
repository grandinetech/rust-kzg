#[cfg(test)]
mod tests {
    use kzg_bench::tests::poly::{
        create_poly_of_length_ten, poly_div_by_zero, poly_div_fast_test, poly_div_long_test,
        poly_div_random, poly_eval_0_check, poly_eval_check, poly_eval_nil_check,
        poly_inverse_simple_0, poly_inverse_simple_1, poly_mul_direct_test, poly_mul_fft_test,
        poly_mul_random, poly_test_div,
    };
    use rust_kzg_zkcrypto::kzg_proofs::FFTSettings;
    use rust_kzg_zkcrypto::kzg_types::ZFr;
    use rust_kzg_zkcrypto::poly::PolyData;

    #[test]
    fn create_poly_of_length_ten_() {
        create_poly_of_length_ten::<ZFr, PolyData>();
    }

    #[test]
    fn poly_eval_check_() {
        poly_eval_check::<ZFr, PolyData>();
    }

    #[test]
    fn poly_eval_0_check_() {
        poly_eval_0_check::<ZFr, PolyData>();
    }

    #[test]
    fn poly_eval_nil_check_() {
        poly_eval_nil_check::<ZFr, PolyData>();
    }

    #[test]
    fn poly_inverse_simple_0_() {
        poly_inverse_simple_0::<ZFr, PolyData>();
    }

    #[test]
    fn poly_inverse_simple_1_() {
        poly_inverse_simple_1::<ZFr, PolyData>();
    }

    #[test]
    fn poly_test_div_() {
        poly_test_div::<ZFr, PolyData>();
    }

    #[test]
    #[should_panic]
    fn poly_div_by_zero_() {
        poly_div_by_zero::<ZFr, PolyData>();
    }

    #[test]
    fn poly_mul_direct_test_() {
        poly_mul_direct_test::<ZFr, PolyData>();
    }

    #[test]
    fn poly_mul_fft_test_() {
        poly_mul_fft_test::<ZFr, PolyData, FFTSettings>();
    }

    #[test]
    fn poly_mul_random_() {
        poly_mul_random::<ZFr, PolyData, FFTSettings>();
    }

    #[test]
    fn poly_div_random_() {
        poly_div_random::<ZFr, PolyData>();
    }

    #[test]
    fn poly_div_long_test_() {
        poly_div_long_test::<ZFr, PolyData>()
    }

    #[test]
    fn poly_div_fast_test_() {
        poly_div_fast_test::<ZFr, PolyData>()
    }
}

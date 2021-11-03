#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::poly::{
        create_poly_of_length_ten, poly_eval_0_check, poly_eval_check, poly_eval_nil_check,
        poly_inverse_simple_0, poly_inverse_simple_1, poly_test_div, poly_div_by_zero, 
        poly_mul_direct_test, poly_mul_fft_test, poly_mul_random, poly_div_random
    };
    use arkworks::kzg_types::FsFr;
    use arkworks::utils::PolyData;
    use arkworks::kzg_proofs::FFTSettings;


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
        poly_mul_fft_test::<FsFr, PolyData, FFTSettings, PolyData>();
    }

    #[test]
    fn poly_mul_random_() {
        poly_mul_random::<FsFr, PolyData, FFTSettings, PolyData>();
    }

    #[test]
    fn poly_div_random_() {
        poly_div_random::<FsFr, PolyData>();
    }
}

#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::poly::{create_poly_of_length_ten, poly_div_random, poly_eval_0_check, poly_eval_check, poly_eval_nil_check, poly_inverse_simple_0, poly_inverse_simple_1, poly_mul_direct_test, poly_mul_fft_test, poly_mul_random, poly_test_div, test_poly_div_by_zero};
    use kzg_from_scratch::kzg_types::{FsFr, FsPoly};

    #[test]
    fn create_poly_of_length_ten_() {
        create_poly_of_length_ten::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_eval_check_() {
        poly_eval_check::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_eval_0_check_() {
        poly_eval_0_check::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_eval_nil_check_() {
        poly_eval_nil_check::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_inverse_simple_0_() {
        poly_inverse_simple_0::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_inverse_simple_1_() {
        poly_inverse_simple_1::<FsFr, FsPoly>()
    }

    #[test]
    fn test_poly_div_by_zero_() {
        test_poly_div_by_zero::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_test_div_() {
        poly_test_div::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_mul_direct_test_() {
        poly_mul_direct_test::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_mul_fft_test_() {
        poly_mul_fft_test::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_mul_random_() {
        // poly_mul_random::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_div_random_() {
        // poly_div_random::<FsFr, FsPoly>()
    }
}
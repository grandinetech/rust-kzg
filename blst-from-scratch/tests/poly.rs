#[path = "./local_tests/local_poly.rs"]
pub mod local_poly;

#[cfg(test)]
pub mod tests {
    use crate::local_poly::{poly_pad_works_rand, create_poly_of_length_ten, poly_div_fast_test, poly_div_long_test, poly_div_random, poly_eval_0_check, poly_eval_check, poly_eval_nil_check, poly_inverse_simple_0, poly_inverse_simple_1, poly_mul_direct_test, poly_mul_fft_test, poly_mul_random, test_poly_div_by_zero};
    use blst_from_scratch::types::fr::FsFr;
    use blst_from_scratch::types::poly::StupidPoly;

    #[test]
    fn create_poly_of_length_ten_() {
        create_poly_of_length_ten::<FsFr, StupidPoly>()
    }

    #[test]
    fn poly_pad_works_rand_() {
        poly_pad_works_rand::<FsFr, StupidPoly>()
    }

    #[test]
    fn poly_eval_check_() {
        poly_eval_check::<FsFr, StupidPoly>()
    }

    #[test]
    fn poly_eval_0_check_() {
        poly_eval_0_check::<FsFr, StupidPoly>()
    }

    #[test]
    fn poly_eval_nil_check_() {
        poly_eval_nil_check::<FsFr, StupidPoly>()
    }

    #[test]
    fn poly_inverse_simple_0_() {
        poly_inverse_simple_0::<FsFr, StupidPoly>()
    }

    #[test]
    fn poly_inverse_simple_1_() {
        poly_inverse_simple_1::<FsFr, StupidPoly>()
    }

    #[test]
    fn test_poly_div_by_zero_() {
        test_poly_div_by_zero::<FsFr, StupidPoly>()
    }

    #[test]
    fn poly_div_long_test_() {
        poly_div_long_test::<FsFr, StupidPoly>()
    }

    #[test]
    fn poly_div_fast_test_() {
        poly_div_fast_test::<FsFr, StupidPoly>()
    }

    #[test]
    fn poly_mul_direct_test_() {
        poly_mul_direct_test::<FsFr, StupidPoly>()
    }

    #[test]
    fn poly_mul_fft_test_() {
        poly_mul_fft_test::<FsFr, StupidPoly>()
    }

    #[test]
    fn poly_mul_random_() {
        poly_mul_random::<FsFr, StupidPoly>()
    }

    #[test]
    fn poly_div_random_() {
        poly_div_random::<FsFr, StupidPoly>()
    }
}
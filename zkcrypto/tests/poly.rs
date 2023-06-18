#[cfg(test)]
mod tests {
    use kzg_bench::tests::poly::*;
    use rust_kzg_zkcrypto::fftsettings::ZkFFTSettings;
    use rust_kzg_zkcrypto::poly::ZPoly;
    use rust_kzg_zkcrypto::zkfr::blsScalar;

    #[test]
    fn create_poly_of_length_ten_() {
        create_poly_of_length_ten::<blsScalar, ZPoly>();
    }

    #[test]
    fn poly_eval_check_() {
        poly_eval_check::<blsScalar, ZPoly>();
    }

    #[test]
    fn poly_eval_0_check_() {
        poly_eval_0_check::<blsScalar, ZPoly>();
    }

    #[test]
    fn poly_eval_nil_check_() {
        poly_eval_nil_check::<blsScalar, ZPoly>();
    }

    #[test]
    fn poly_inverse_simple_0_() {
        poly_inverse_simple_0::<blsScalar, ZPoly>();
    }

    #[test]
    fn poly_inverse_simple_1_() {
        poly_inverse_simple_1::<blsScalar, ZPoly>();
    }

    #[test]
    pub fn poly_test_div_() {
        poly_test_div::<blsScalar, ZPoly>();
    }

    #[test]
    pub fn poly_div_by_zero_() {
        poly_div_by_zero::<blsScalar, ZPoly>();
    }

    #[test]
    pub fn poly_mul_direct_test_() {
        poly_mul_direct_test::<blsScalar, ZPoly>();
    }

    #[test]
    pub fn poly_mul_fft_test_() {
        poly_mul_fft_test::<blsScalar, ZPoly, ZkFFTSettings>();
    }

    #[test]
    pub fn poly_mul_random_() {
        poly_mul_random::<blsScalar, ZPoly, ZkFFTSettings>();
    }

    #[test]
    pub fn poly_div_random_() {
        poly_div_random::<blsScalar, ZPoly>();
    }

    #[test]
    fn poly_div_long_test_() {
        poly_div_long_test::<blsScalar, ZPoly>()
    }

    #[test]
    fn poly_div_fast_test_() {
        poly_div_fast_test::<blsScalar, ZPoly>()
    }
}

#[cfg(test)]
mod tests {
    use kzg_bench::tests::poly::*;
    use kzg_bindings::fftsettings::KzgFFTSettings;
    use kzg_bindings::finite::BlstFr;
    use kzg_bindings::poly::KzgPoly;

    #[test]
    fn test_create_poly_of_length_ten() {
        create_poly_of_length_ten::<BlstFr, KzgPoly>();
    }

    #[test]
    fn test_poly_eval_check() {
        poly_eval_check::<BlstFr, KzgPoly>();
    }

    #[test]
    fn test_poly_eval_0_check() {
        poly_eval_0_check::<BlstFr, KzgPoly>();
    }

    #[test]
    fn test_poly_eval_nil_check() {
        poly_eval_nil_check::<BlstFr, KzgPoly>();
    }

    #[test]
    fn test_poly_inverse_simple_0() {
        poly_inverse_simple_0::<BlstFr, KzgPoly>();
    }

    #[test]
    fn test_poly_inverse_simple_1() {
        poly_inverse_simple_1::<BlstFr, KzgPoly>();
    }

    #[test]
    fn test_poly_test_div() {
        poly_test_div::<BlstFr, KzgPoly>();
    }

    #[test]
    fn test_poly_div_by_zero() {
        poly_div_by_zero::<BlstFr, KzgPoly>();
    }

    #[test]
    fn test_poly_mul_direct_test() {
        poly_mul_direct_test::<BlstFr, KzgPoly>();
    }

    #[test]
    fn test_poly_mul_fft_test() {
        poly_mul_fft_test::<BlstFr, KzgPoly, KzgFFTSettings>();
    }

    #[test]
    fn test_poly_mul_random() {
        //poly_mul_random::<BlstFr, KzgPoly, KzgFFTSettings>();
    }

    #[test]
    fn test_poly_div_random() {
        poly_div_random::<BlstFr, KzgPoly>();
    }
}

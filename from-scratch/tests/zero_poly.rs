#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::zero_poly::{check_test_data, reduce_partials_random, test_reduce_partials, zero_poly_252, zero_poly_all_but_one, zero_poly_known, zero_poly_random};
    use kzg_from_scratch::fft_fr::fft_fr;
    use kzg_from_scratch::kzg_types::{FFTSettings, Fr, Poly};
    use kzg_from_scratch::zero_poly::{do_zero_poly_mul_partial, reduce_partials, zero_poly_via_multiplication};

    #[test]
    fn test_reduce_partials_() {
        test_reduce_partials(&do_zero_poly_mul_partial, &reduce_partials);
    }

    #[test]
    fn reduce_partials_random_() {
        reduce_partials_random(&do_zero_poly_mul_partial, &reduce_partials);
    }

    #[test]
    fn check_test_data_() {
        check_test_data::<Fr, FFTSettings, Poly>(&fft_fr);
    }

    #[test]
    fn zero_poly_known_() {
        zero_poly_known(&zero_poly_via_multiplication);
    }

    #[test]
    fn zero_poly_random_() {
        zero_poly_random(&fft_fr, &zero_poly_via_multiplication);
    }

    #[test]
    fn zero_poly_all_but_one_() {
        zero_poly_all_but_one(&fft_fr, &zero_poly_via_multiplication);
    }

    #[test]
    fn zero_poly_252_() {
        zero_poly_252(&fft_fr, &zero_poly_via_multiplication);
    }
}
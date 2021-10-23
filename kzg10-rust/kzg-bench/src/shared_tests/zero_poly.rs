#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::zero_poly::{check_test_data, reduce_partials_random, test_reduce_partials, zero_poly_252, zero_poly_all_but_one, zero_poly_known, zero_poly_random};
    use mcl_rust::kzg10::Polynomial;
    use mcl_rust::fk20_fft::FFTSettings;
    use mcl_rust::data_types::fr::Fr;

    #[test]
    fn test_reduce_partials_() {
        test_reduce_partials::<Fr, FFTSettings, Polynomial>();
    }

    #[test]
    fn reduce_partials_random_() {
        reduce_partials_random::<Fr, FFTSettings, Polynomial>();
    }

    #[test]
    fn check_test_data_() {
        check_test_data::<Fr, FFTSettings, Polynomial>();
    }

    #[test]
    fn zero_poly_known_() {
        zero_poly_known::<Fr, FFTSettings, Polynomial>();
    }

    #[test]
    fn zero_poly_random_() {
        zero_poly_random::<Fr, FFTSettings, Polynomial>();
    }

    #[test]
    fn zero_poly_all_but_one_() {
        zero_poly_all_but_one::<Fr, FFTSettings, Polynomial>();
    }

    #[test]
    fn zero_poly_252_() {
        zero_poly_252::<Fr, FFTSettings, Polynomial>();
    }
}
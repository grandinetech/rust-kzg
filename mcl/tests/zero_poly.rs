#[cfg(test)]
mod tests {
    use kzg_bench::tests::zero_poly::{
        check_test_data, reduce_partials_random, test_reduce_partials, zero_poly_252,
        zero_poly_all_but_one, zero_poly_known, zero_poly_random,
    };
    use rust_kzg_mcl::types::fft_settings::MclFFTSettings;
    use rust_kzg_mcl::types::fr::MclFr;
    use rust_kzg_mcl::types::poly::MclPoly;

    #[test]
    fn test_reduce_partials_() {
        test_reduce_partials::<MclFr, MclFFTSettings, MclPoly>();
    }

    #[test]
    fn reduce_partials_random_() {
        reduce_partials_random::<MclFr, MclFFTSettings, MclPoly>();
    }

    #[test]
    fn check_test_data_() {
        check_test_data::<MclFr, MclFFTSettings, MclPoly>();
    }

    #[test]
    fn zero_poly_known_() {
        zero_poly_known::<MclFr, MclFFTSettings, MclPoly>();
    }

    #[test]
    fn zero_poly_random_() {
        zero_poly_random::<MclFr, MclFFTSettings, MclPoly>();
    }

    #[test]
    fn zero_poly_all_but_one_() {
        zero_poly_all_but_one::<MclFr, MclFFTSettings, MclPoly>();
    }

    #[test]
    fn zero_poly_252_() {
        zero_poly_252::<MclFr, MclFFTSettings, MclPoly>();
    }
}

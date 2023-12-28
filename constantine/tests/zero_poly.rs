#[cfg(test)]
mod tests {
    use kzg_bench::tests::zero_poly::{
        check_test_data, reduce_partials_random, test_reduce_partials, zero_poly_252,
        zero_poly_all_but_one, zero_poly_known, zero_poly_random,
    };
    use rust_kzg_constantine::types::fft_settings::CtFFTSettings;
    use rust_kzg_constantine::types::fr::CtFr;
    use rust_kzg_constantine::types::poly::CtPoly;

    #[test]
    fn test_reduce_partials_() {
        test_reduce_partials::<CtFr, CtFFTSettings, CtPoly>();
    }

    #[test]
    fn reduce_partials_random_() {
        reduce_partials_random::<CtFr, CtFFTSettings, CtPoly>();
    }

    #[test]
    fn check_test_data_() {
        check_test_data::<CtFr, CtFFTSettings, CtPoly>();
    }

    #[test]
    fn zero_poly_known_() {
        zero_poly_known::<CtFr, CtFFTSettings, CtPoly>();
    }

    #[test]
    fn zero_poly_random_() {
        zero_poly_random::<CtFr, CtFFTSettings, CtPoly>();
    }

    #[test]
    fn zero_poly_all_but_one_() {
        zero_poly_all_but_one::<CtFr, CtFFTSettings, CtPoly>();
    }

    #[test]
    fn zero_poly_252_() {
        zero_poly_252::<CtFr, CtFFTSettings, CtPoly>();
    }
}

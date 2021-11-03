#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::zero_poly::{
        check_test_data, reduce_partials_random, test_reduce_partials, zero_poly_252,
        zero_poly_all_but_one, zero_poly_known, zero_poly_random,
    };
    use arkworks::kzg_proofs::FFTSettings;
    use arkworks::kzg_types::FsFr;
    use arkworks::utils::PolyData;

    #[test]
    fn test_reduce_partials_() {
        test_reduce_partials::<FsFr, FFTSettings, PolyData>();
    }

    #[test]
    fn reduce_partials_random_() {
        reduce_partials_random::<FsFr, FFTSettings, PolyData>();
    }

    #[test]
    fn check_test_data_() {
        check_test_data::<FsFr, FFTSettings, PolyData>();
    }

    #[test]
    fn zero_poly_known_() {
        zero_poly_known::<FsFr, FFTSettings, PolyData>();
    }

    #[test]
    fn zero_poly_random_() {
        zero_poly_random::<FsFr, FFTSettings, PolyData>();
    }

    #[test]
    fn zero_poly_all_but_one_() {
        zero_poly_all_but_one::<FsFr, FFTSettings, PolyData>();
    }

    #[test]
    fn zero_poly_252_() {
        zero_poly_252::<FsFr, FFTSettings, PolyData>();
    }
}

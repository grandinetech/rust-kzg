#[cfg(test)]
mod tests {
    use kzg_bench::tests::zero_poly::{
        check_test_data, reduce_partials_random, test_reduce_partials, zero_poly_252,
        zero_poly_all_but_one, zero_poly_known, zero_poly_random,
    };
    use rust_kzg_arkworks3::kzg_proofs::FFTSettings;
    use rust_kzg_arkworks3::kzg_types::ArkFr;
    use rust_kzg_arkworks3::utils::PolyData;

    #[test]
    fn test_reduce_partials_() {
        test_reduce_partials::<ArkFr, FFTSettings, PolyData>();
    }

    #[test]
    fn reduce_partials_random_() {
        reduce_partials_random::<ArkFr, FFTSettings, PolyData>();
    }

    #[test]
    fn check_test_data_() {
        check_test_data::<ArkFr, FFTSettings, PolyData>();
    }

    #[test]
    fn zero_poly_known_() {
        zero_poly_known::<ArkFr, FFTSettings, PolyData>();
    }

    #[test]
    fn zero_poly_random_() {
        zero_poly_random::<ArkFr, FFTSettings, PolyData>();
    }

    #[test]
    fn zero_poly_all_but_one_() {
        zero_poly_all_but_one::<ArkFr, FFTSettings, PolyData>();
    }

    #[test]
    fn zero_poly_252_() {
        zero_poly_252::<ArkFr, FFTSettings, PolyData>();
    }
}

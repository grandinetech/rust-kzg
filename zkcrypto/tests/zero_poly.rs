#[cfg(test)]
mod tests {
    use kzg_bench::tests::zero_poly::{
        check_test_data, reduce_partials_random, test_reduce_partials, zero_poly_252,
        zero_poly_all_but_one, zero_poly_known, zero_poly_random,
    };
    use rust_kzg_zkcrypto::kzg_proofs::FFTSettings;
    use rust_kzg_zkcrypto::kzg_types::ZFr;
    use rust_kzg_zkcrypto::poly::PolyData;

    #[test]
    fn test_reduce_partials_() {
        test_reduce_partials::<ZFr, FFTSettings, PolyData>();
    }

    #[test]
    fn reduce_partials_random_() {
        reduce_partials_random::<ZFr, FFTSettings, PolyData>();
    }

    #[test]
    fn check_test_data_() {
        check_test_data::<ZFr, FFTSettings, PolyData>();
    }

    #[test]
    fn zero_poly_known_() {
        zero_poly_known::<ZFr, FFTSettings, PolyData>();
    }

    #[test]
    fn zero_poly_random_() {
        zero_poly_random::<ZFr, FFTSettings, PolyData>();
    }

    #[test]
    fn zero_poly_all_but_one_() {
        zero_poly_all_but_one::<ZFr, FFTSettings, PolyData>();
    }

    #[test]
    fn zero_poly_252_() {
        zero_poly_252::<ZFr, FFTSettings, PolyData>();
    }
}

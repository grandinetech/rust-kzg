#[cfg(test)]
mod tests {
    use kzg_bench::tests::zero_poly::{
        check_test_data, reduce_partials_random, test_reduce_partials, zero_poly_252,
        zero_poly_all_but_one, zero_poly_known, zero_poly_random,
    };
    use rust_kzg_zkcrypto::fftsettings::ZkFFTSettings;
    use rust_kzg_zkcrypto::poly::ZPoly;
    use rust_kzg_zkcrypto::zkfr::blsScalar;

    #[test]
    fn test_reduce_partials_() {
        test_reduce_partials::<blsScalar, ZkFFTSettings, ZPoly>();
    }

    #[test]
    fn reduce_partials_random_() {
        reduce_partials_random::<blsScalar, ZkFFTSettings, ZPoly>();
    }

    #[test]
    fn check_test_data_() {
        check_test_data::<blsScalar, ZkFFTSettings, ZPoly>();
    }

    #[test]
    fn zero_poly_known_() {
        zero_poly_known::<blsScalar, ZkFFTSettings, ZPoly>();
    }

    #[test]
    fn zero_poly_random_() {
        zero_poly_random::<blsScalar, ZkFFTSettings, ZPoly>();
    }

    #[test]
    fn zero_poly_all_but_one_() {
        zero_poly_all_but_one::<blsScalar, ZkFFTSettings, ZPoly>();
    }

    #[test]
    fn zero_poly_252_() {
        zero_poly_252::<blsScalar, ZkFFTSettings, ZPoly>();
    }
}

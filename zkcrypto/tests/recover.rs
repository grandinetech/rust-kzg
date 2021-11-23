#[cfg(test)]
pub mod recover_tests {
    use kzg_bench::tests::recover::*;
    use zkcrypto::zkfr::blsScalar;
    use zkcrypto::poly::{ZPoly};
    use zkcrypto::fftsettings::ZkFFTSettings;

    #[test]
    fn recover_simple_() {
        recover_simple::<blsScalar, ZkFFTSettings, ZPoly, ZPoly>();
    }

    #[test]
    fn recover_random_() {
        recover_random::<blsScalar, ZkFFTSettings, ZPoly, ZPoly>();
    }
}

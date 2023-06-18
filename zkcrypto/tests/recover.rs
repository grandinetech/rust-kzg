#[cfg(test)]
mod recover_tests {
    use kzg_bench::tests::recover::*;
    use rust_kzg_zkcrypto::fftsettings::ZkFFTSettings;
    use rust_kzg_zkcrypto::poly::ZPoly;
    use rust_kzg_zkcrypto::zkfr::blsScalar;

    #[test]
    fn recover_simple_() {
        recover_simple::<blsScalar, ZkFFTSettings, ZPoly, ZPoly>();
    }

    #[test]
    fn recover_random_() {
        recover_random::<blsScalar, ZkFFTSettings, ZPoly, ZPoly>();
    }

    #[test]
    fn more_than_half_missing_() {
        more_than_half_missing::<blsScalar, ZkFFTSettings, ZPoly, ZPoly>();
    }
}

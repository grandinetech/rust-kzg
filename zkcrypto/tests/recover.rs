#[cfg(test)]
mod recover_tests {
    use kzg_bench::tests::recover::*;
    use zkcrypto::fftsettings::ZkFFTSettings;
    use zkcrypto::poly::ZPoly;
    use zkcrypto::zkfr::blsScalar;

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

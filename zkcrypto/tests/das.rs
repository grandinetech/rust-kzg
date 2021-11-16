#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::das::{das_extension_test_known, das_extension_test_random};
    use zkcrypto::zkfr::blsScalar;
    use zkcrypto::fftsettings::ZkFFTSettings;

    #[test]
    fn das_extension_test_known_() {
        das_extension_test_known::<blsScalar, ZkFFTSettings>();
    }

    #[test]
    fn das_extension_test_random_() {
        das_extension_test_random::<blsScalar, ZkFFTSettings>();
    }
}
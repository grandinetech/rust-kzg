#[cfg(test)]
mod tests {
    use kzg_bench::tests::das::{das_extension_test_known, das_extension_test_random};
    use rust_kzg_zkcrypto::fftsettings::ZkFFTSettings;
    use rust_kzg_zkcrypto::zkfr::blsScalar;

    #[test]
    fn das_extension_test_known_() {
        das_extension_test_known::<blsScalar, ZkFFTSettings>();
    }

    #[test]
    fn das_extension_test_random_() {
        das_extension_test_random::<blsScalar, ZkFFTSettings>();
    }
}

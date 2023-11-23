#[cfg(test)]
mod tests {
    use kzg_bench::tests::das::{das_extension_test_known, das_extension_test_random};
    use rust_kzg_zkcrypto::kzg_proofs::FFTSettings;
    use rust_kzg_zkcrypto::kzg_types::ZFr;

    #[test]
    fn das_extension_test_known_() {
        das_extension_test_known::<ZFr, FFTSettings>();
    }

    #[test]
    fn das_extension_test_random_() {
        das_extension_test_random::<ZFr, FFTSettings>();
    }
}

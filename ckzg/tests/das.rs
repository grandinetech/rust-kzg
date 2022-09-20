#[cfg(test)]
pub mod tests {
    use ckzg::fftsettings::KzgFFTSettings;
    use ckzg::finite::BlstFr;
    use kzg_bench::tests::das::{das_extension_test_known, das_extension_test_random};

    #[test]
    fn das_extension_test_known_() {
        das_extension_test_known::<BlstFr, KzgFFTSettings>();
    }

    #[test]
    fn das_extension_test_random_() {
        das_extension_test_random::<BlstFr, KzgFFTSettings>();
    }
}

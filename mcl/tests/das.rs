#[cfg(test)]
mod tests {
    use kzg_bench::tests::das::{das_extension_test_known, das_extension_test_random};
    use rust_kzg_mcl::types::fft_settings::MclFFTSettings;
    use rust_kzg_mcl::types::fr::MclFr;

    #[test]
    fn das_extension_test_known_() {
        das_extension_test_known::<MclFr, MclFFTSettings>();
    }

    #[test]
    fn das_extension_test_random_() {
        das_extension_test_random::<MclFr, MclFFTSettings>();
    }
}

#[cfg(test)]
mod tests {
    use kzg_bench::tests::das::{das_extension_test_known, das_extension_test_random};
    use rust_kzg_blst::types::fft_settings::FsFFTSettings;
    use rust_kzg_blst::types::fr::FsFr;

    #[test]
    fn das_extension_test_known_() {
        das_extension_test_known::<FsFr, FsFFTSettings>();
    }

    #[test]
    fn das_extension_test_random_() {
        das_extension_test_random::<FsFr, FsFFTSettings>();
    }
}

#[cfg(test)]
mod tests {
    use kzg_bench::tests::das::{das_extension_test_known, das_extension_test_random};
    use rust_kzg_constantine::types::fft_settings::CtFFTSettings;
    use rust_kzg_constantine::types::fr::CtFr;

    #[test]
    fn das_extension_test_known_() {
        das_extension_test_known::<CtFr, CtFFTSettings>();
    }

    #[test]
    fn das_extension_test_random_() {
        das_extension_test_random::<CtFr, CtFFTSettings>();
    }
}

#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::das::{das_extension_test_known, das_extension_test_random};
    use kzg_from_scratch::das::das_fft_extension;
    use kzg_from_scratch::fft_fr::fft_fr;

    #[test]
    fn das_extension_test_known_() {
        das_extension_test_known(&das_fft_extension);
    }

    #[test]
    fn das_extension_test_random_() {
        das_extension_test_random(&das_fft_extension, &fft_fr);
    }
}
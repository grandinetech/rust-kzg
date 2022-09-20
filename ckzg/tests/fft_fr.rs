#[cfg(test)]
mod tests {
    use ckzg::fftsettings::{bound_fft_fr_fast, bound_fft_fr_slow, KzgFFTSettings};
    use ckzg::finite::BlstFr;
    use kzg_bench::tests::fft_fr::{compare_sft_fft, inverse_fft, roundtrip_fft, stride_fft};

    #[test]
    fn test_roundtrip_fft() {
        roundtrip_fft::<BlstFr, KzgFFTSettings>();
    }

    #[test]
    fn test_inverse_fft() {
        inverse_fft::<BlstFr, KzgFFTSettings>();
    }

    #[test]
    fn test_stride_fft() {
        stride_fft::<BlstFr, KzgFFTSettings>();
    }

    #[test]
    fn test_compare_sft_fft() {
        compare_sft_fft::<BlstFr, KzgFFTSettings>(&bound_fft_fr_slow, &bound_fft_fr_fast);
    }
}

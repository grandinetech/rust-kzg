#[cfg(test)]
mod tests {
    use kzg_bench::tests::fft_fr::{roundtrip_fft, inverse_fft, stride_fft, compare_sft_fft};
    use ckzg::fftsettings::{KzgFFTSettings, bound_fft_fr_slow, bound_fft_fr_fast};
    use ckzg::finite::BlstFr;

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

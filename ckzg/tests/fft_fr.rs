#[cfg(test)]
mod tests {
    use kzg_bench::tests::fft_fr::{roundtrip_fft, inverse_fft, stride_fft};
    use kzg_bindings::fftsettings::KzgFFTSettings;
    use kzg_bindings::finite::BlstFr;

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
}

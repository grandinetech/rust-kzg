#[cfg(test)]
mod tests {
    use kzg_bench::tests::fft_g1::{roundtrip_fft, stride_fft};
    use kzg_bindings::fftsettings::{KzgFFTSettings, make_data};
    use kzg_bindings::consts::BlstP1;
    use kzg_bindings::finite::BlstFr;

    #[test]
    fn test_roundtrip_fft() {
        roundtrip_fft::<BlstFr, BlstP1, KzgFFTSettings>(&make_data);
    }

    #[test]
    fn test_stride_fft() {
        stride_fft::<BlstFr, BlstP1, KzgFFTSettings>(&make_data);
    }
}

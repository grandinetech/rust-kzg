#[cfg(test)]
mod tests {
    use kzg_bench::tests::fft_g1::{roundtrip_fft, stride_fft, compare_sft_fft};
    use ckzg::fftsettings::{KzgFFTSettings, make_data, bound_fft_g1_slow, bound_fft_g1_fast};
    use ckzg::consts::BlstP1;
    use ckzg::finite::BlstFr;

    #[test]
    fn test_roundtrip_fft() {
        roundtrip_fft::<BlstFr, BlstP1, KzgFFTSettings>(&make_data);
    }

    #[test]
    fn test_stride_fft() {
        stride_fft::<BlstFr, BlstP1, KzgFFTSettings>(&make_data);
    }

    #[test]
    fn test_compare_sft_fft() {
        compare_sft_fft::<BlstFr, BlstP1, KzgFFTSettings>(&bound_fft_g1_slow, &bound_fft_g1_fast, &make_data);
    }
}

#[cfg(test)]
mod tests {
    use ckzg::consts::BlstP1;
    use ckzg::fftsettings::{bound_fft_g1_fast, bound_fft_g1_slow, make_data, KzgFFTSettings};
    use ckzg::finite::BlstFr;
    use kzg_bench::tests::fft_g1::{compare_sft_fft, roundtrip_fft, stride_fft};

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
        compare_sft_fft::<BlstFr, BlstP1, KzgFFTSettings>(
            &bound_fft_g1_slow,
            &bound_fft_g1_fast,
            &make_data,
        );
    }
}

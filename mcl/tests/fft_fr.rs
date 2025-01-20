#[cfg(test)]
mod tests {
    use kzg_bench::tests::fft_fr::{compare_sft_fft, inverse_fft, roundtrip_fft, stride_fft};
    use rust_kzg_mcl::fft_fr::{fft_fr_fast, fft_fr_slow};
    use rust_kzg_mcl::types::fft_settings::MclFFTSettings;
    use rust_kzg_mcl::types::fr::MclFr;

    #[test]
    fn compare_sft_fft_() {
        compare_sft_fft::<MclFr, MclFFTSettings>(&fft_fr_slow, &fft_fr_fast);
    }

    #[test]
    fn roundtrip_fft_() {
        roundtrip_fft::<MclFr, MclFFTSettings>();
    }

    #[test]
    fn inverse_fft_() {
        inverse_fft::<MclFr, MclFFTSettings>();
    }

    #[test]
    fn stride_fft_() {
        stride_fft::<MclFr, MclFFTSettings>();
    }
}

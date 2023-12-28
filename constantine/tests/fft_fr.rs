#[cfg(test)]
mod tests {
    use kzg_bench::tests::fft_fr::{compare_sft_fft, inverse_fft, roundtrip_fft, stride_fft};
    use rust_kzg_constantine::fft_fr::{fft_fr_fast, fft_fr_slow};
    use rust_kzg_constantine::types::fft_settings::CtFFTSettings;
    use rust_kzg_constantine::types::fr::CtFr;

    #[test]
    fn compare_sft_fft_() {
        compare_sft_fft::<CtFr, CtFFTSettings>(&fft_fr_slow, &fft_fr_fast);
    }

    #[test]
    fn roundtrip_fft_() {
        roundtrip_fft::<CtFr, CtFFTSettings>();
    }

    #[test]
    fn inverse_fft_() {
        inverse_fft::<CtFr, CtFFTSettings>();
    }

    #[test]
    fn stride_fft_() {
        stride_fft::<CtFr, CtFFTSettings>();
    }
}

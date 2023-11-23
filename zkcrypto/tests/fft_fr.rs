#[cfg(test)]
mod tests {
    use kzg_bench::tests::fft_fr::{compare_sft_fft, inverse_fft, roundtrip_fft, stride_fft};
    use rust_kzg_zkcrypto::fft::{fft_fr_fast, fft_fr_slow};
    use rust_kzg_zkcrypto::kzg_proofs::FFTSettings;
    use rust_kzg_zkcrypto::kzg_types::ZFr;

    #[test]
    fn compare_sft_fft_() {
        compare_sft_fft::<ZFr, FFTSettings>(&fft_fr_slow, &fft_fr_fast);
    }

    #[test]
    fn roundtrip_fft_() {
        roundtrip_fft::<ZFr, FFTSettings>();
    }

    #[test]
    fn inverse_fft_() {
        inverse_fft::<ZFr, FFTSettings>();
    }

    #[test]
    fn stride_fft_() {
        stride_fft::<ZFr, FFTSettings>();
    }
}

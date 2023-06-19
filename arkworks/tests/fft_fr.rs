#[cfg(test)]
mod tests {
    use kzg_bench::tests::fft_fr::{compare_sft_fft, inverse_fft, roundtrip_fft, stride_fft};
    use rust_kzg_arkworks::fft::{fft_fr_fast, fft_fr_slow};
    use rust_kzg_arkworks::kzg_proofs::FFTSettings;
    use rust_kzg_arkworks::kzg_types::FsFr;

    #[test]
    fn compare_sft_fft_() {
        compare_sft_fft::<FsFr, FFTSettings>(&fft_fr_slow, &fft_fr_fast);
    }

    #[test]
    fn roundtrip_fft_() {
        roundtrip_fft::<FsFr, FFTSettings>();
    }

    #[test]
    fn inverse_fft_() {
        inverse_fft::<FsFr, FFTSettings>();
    }

    #[test]
    fn stride_fft_() {
        stride_fft::<FsFr, FFTSettings>();
    }
}

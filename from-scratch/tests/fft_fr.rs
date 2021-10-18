#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::fft_fr::{compare_sft_fft, inverse_fft, roundtrip_fft, stride_fft};
    use kzg_from_scratch::fft_fr::{fft_fr, fft_fr_fast, fft_fr_slow};
    use kzg_from_scratch::kzg_types::{FsFFTSettings, FsFr};

    #[test]
    fn compare_sft_fft_() {
        compare_sft_fft::<FsFr, FsFFTSettings>(&fft_fr_slow, &fft_fr_fast);
    }

    #[test]
    fn roundtrip_fft_() {
        roundtrip_fft(&fft_fr);
    }

    #[test]
    fn inverse_fft_() {
        inverse_fft(&fft_fr);
    }

    #[test]
    fn stride_fft_() {
        stride_fft(&fft_fr);
    }
}
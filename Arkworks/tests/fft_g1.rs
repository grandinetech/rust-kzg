#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::fft_g1::{stride_fft, roundtrip_fft, compare_sft_fft};
    use arkworks::fft_g1::{make_data, fft_g1_slow, fft_g1_fast};
    use arkworks::kzg_proofs::FFTSettings;
    use arkworks::kzg_types::{ArkG1, FsFr};

    #[test]
    fn roundtrip_fft_() {
        roundtrip_fft::<FsFr, ArkG1, FFTSettings>(&make_data);
    }

    #[test]
    fn stride_fft_() {
        stride_fft::<FsFr, ArkG1, FFTSettings>(&make_data);
    }

    #[test]
    fn compare_sft_fft_() {
        compare_sft_fft::<FsFr, ArkG1, FFTSettings>(&fft_g1_fast, &fft_g1_slow, &make_data);
    }
}

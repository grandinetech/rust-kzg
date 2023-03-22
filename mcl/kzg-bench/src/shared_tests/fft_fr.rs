#[cfg(test)]
mod fft_fr_tests {
    use kzg_bench::tests::fft_fr::*;
    use mcl_rust::data_types::fr::Fr;
    use mcl_rust::fk20_fft::FFTSettings;
    use mcl_rust::mcl_methods::init;
    use mcl_rust::CurveType;

    #[test]
    fn compare_sft_fft_() {
        assert!(init(CurveType::BLS12_381));
        compare_sft_fft::<Fr, FFTSettings>(&FFTSettings::fft_fr_slow, &FFTSettings::fft_fr_fast);
    }

    #[test]
    fn roundtrip_fft_fr_() {
        assert!(init(CurveType::BLS12_381));
        roundtrip_fft::<Fr, FFTSettings>();
    }

    #[test]
    fn inverse_fft_fr_() {
        assert!(init(CurveType::BLS12_381));
        inverse_fft::<Fr, FFTSettings>();
    }

    #[test]
    fn stride_fft_fr_() {
        assert!(init(CurveType::BLS12_381));
        stride_fft::<Fr, FFTSettings>();
    }
}

#[cfg(test)]
mod fft_fr_tests {
    use kzg_bench::tests::fft_fr::*;
    use rust_kzg_mcl::data_types::fr::Fr;
    use rust_kzg_mcl::fk20_fft::FFTSettings;
    use rust_kzg_mcl::mcl_methods::init;
    use rust_kzg_mcl::CurveType;

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

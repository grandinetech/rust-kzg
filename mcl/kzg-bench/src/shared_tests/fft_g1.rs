#[cfg(test)]
mod fft_g1_tests {
    use kzg_bench::tests::fft_g1::*;
    use rust_kzg_mcl::data_types::fr::Fr;
    use rust_kzg_mcl::data_types::g1::G1;
    use rust_kzg_mcl::fk20_fft::{make_data, FFTSettings};
    use rust_kzg_mcl::mcl_methods::init;
    use rust_kzg_mcl::CurveType;

    #[test]
    fn roundtrip_fft_g1_() {
        assert!(init(CurveType::BLS12_381));
        roundtrip_fft::<Fr, G1, FFTSettings>(&make_data);
    }

    #[test]
    fn stride_fft_g1_() {
        assert!(init(CurveType::BLS12_381));
        stride_fft::<Fr, G1, FFTSettings>(&make_data);
    }

    #[test]
    fn compare_sft_fft_() {
        assert!(init(CurveType::BLS12_381));
        compare_sft_fft::<Fr, G1, FFTSettings>(
            &FFTSettings::fft_g1_slow,
            &FFTSettings::fft_g1_fast,
            &make_data,
        );
    }
}

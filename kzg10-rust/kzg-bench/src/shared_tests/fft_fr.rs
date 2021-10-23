#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::fft_fr::{inverse_fft, roundtrip_fft, stride_fft};
    use mcl_rust::data_types::fr::Fr;
    use mcl_rust::fk20_fft::FFTSettings;
    use mcl_rust::mcl_methods::init;
    use mcl_rust::CurveType;

    #[test]
    #[test]
    fn roundtrip_fft_() {
        assert!(init(CurveType::BLS12_381));
        roundtrip_fft::<Fr, FFTSettings>();
    }

    #[test]
    fn inverse_fft_() {
        assert!(init(CurveType::BLS12_381));
        inverse_fft::<Fr, FFTSettings>();
    }

    #[test]
    fn stride_fft_() {
        assert!(init(CurveType::BLS12_381));
        stride_fft::<Fr, FFTSettings>();
    }
}

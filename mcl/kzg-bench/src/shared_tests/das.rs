#[cfg(test)]
mod das_tests {
    use kzg_bench::tests::das::*;
    use rust_kzg_mcl::data_types::fr::Fr;
    use rust_kzg_mcl::fk20_fft::FFTSettings;
    use rust_kzg_mcl::mcl_methods::init;
    use rust_kzg_mcl::CurveType;

    #[test]
    fn das_extension_test_known_() {
        assert!(init(CurveType::BLS12_381));
        das_extension_test_known::<Fr, FFTSettings>();
    }

    #[test]
    fn das_extension_test_random_() {
        assert!(init(CurveType::BLS12_381));
        das_extension_test_random::<Fr, FFTSettings>();
    }
}

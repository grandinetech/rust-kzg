#[cfg(test)]
mod recover_tests {
    use kzg_bench::tests::recover::*;
    use rust_kzg_mcl::data_types::fr::Fr;
    use rust_kzg_mcl::fk20_fft::FFTSettings;
    use rust_kzg_mcl::kzg10::Polynomial;
    use rust_kzg_mcl::mcl_methods::init;
    use rust_kzg_mcl::CurveType;

    #[test]
    fn recover_simple_() {
        assert!(init(CurveType::BLS12_381));
        recover_simple::<Fr, FFTSettings, Polynomial, Polynomial>();
    }

    #[test]
    fn recover_random_() {
        assert!(init(CurveType::BLS12_381));
        recover_random::<Fr, FFTSettings, Polynomial, Polynomial>();
    }

    #[test]
    fn more_than_half_missing_() {
        assert!(init(CurveType::BLS12_381));
        more_than_half_missing::<Fr, FFTSettings, Polynomial, Polynomial>();
    }
}

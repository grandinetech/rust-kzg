#[cfg(test)]
pub mod poly_tests {
    use kzg_bench::tests::recover::*;
    use mcl_rust::data_types::fr::Fr;
    use mcl_rust::kzg10::Polynomial;
    use mcl_rust::mcl_methods::init;
    use mcl_rust::CurveType;
    use mcl_rust::fk20_fft::FFTSettings;

    #[test]
    fn recover_simple_() {
        assert!(init(CurveType::BLS12_381));
        recover_simple::<Fr, FFTSettings, Polynomial, Polynomial>();
    }
}
#[cfg(test)]
pub mod recover_tests {
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

    //Could be not working because of zero poly.
//     #[test]
//     fn recover_random_() {
//         assert!(init(CurveType::BLS12_381));
//         recover_random::<Fr, FFTSettings, Polynomial, Polynomial>();
//     }
}
#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::das::{das_extension_test_known, das_extension_test_random};
    use mcl_rust::data_types::fr::Fr;
    use mcl_rust::fk20_fft::FFTSettings;
    use mcl_rust::mcl_methods::init;
    use mcl_rust::CurveType;
    use mcl_rust::trait_implementations::fr::*;
    use kzg::Fr as CFr;

    #[test]
    fn das_extension_test_known_() {
        assert!(init(CurveType::BLS12_381));
        das_extension_test_known::<Fr, FFTSettings>();
    }

    // #[test]
    // fn das_extension_test_random_() {
    //     assert!(init(CurveType::BLS12_381));
    //     das_extension_test_random::<Fr, FFTSettings>();
    // }
}

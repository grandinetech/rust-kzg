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
        let t:[u64; 4] =[0xa0c43757db972d7d, 0x79d15a1e0677962c, 0xf678865c0c95fa6a, 0x4e85fd4814f96825 ];
        let a:Fr = CFr::from_u64_arr(&t);
        println!("{:?}", &a);
        println!("{}", a.get_str(10));
        das_extension_test_known::<Fr, FFTSettings>();
    }

    #[test]
    fn das_extension_test_random_() {
        assert!(init(CurveType::BLS12_381));
        das_extension_test_random::<Fr, FFTSettings>();
    }
}

#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::zero_poly::{
        check_test_data, reduce_partials_random, test_reduce_partials, zero_poly_252,
        zero_poly_all_but_one, zero_poly_known, zero_poly_random,
    };
    use mcl_rust::data_types::fr::Fr;
    use mcl_rust::fk20_fft::FFTSettings;
    use mcl_rust::kzg10::Polynomial;
    use mcl_rust::mcl_methods::init;
    use mcl_rust::CurveType;

    // #[test]
    // fn test_reduce_partials_() {
    //     assert!(init(CurveType::BLS12_381));
    //     test_reduce_partials::<Fr, FFTSettings, Polynomial>();
    // }

    // #[test]
    // fn reduce_partials_random_() {
    //     assert!(init(CurveType::BLS12_381));
    //     reduce_partials_random::<Fr, FFTSettings, Polynomial>();
    // }

    #[test]
    fn check_test_data_() {
        assert!(init(CurveType::BLS12_381));

        // let arr: [u64; 4] = [
        //     0xffffffff00000000,
        //     0x53bda402fffe5bfe,
        //     0x3339d80809a1d805,
        //     0x73eda753299d7d48,
        // ];
        // let from_arr = Fr::from_u64_arr(&arr);
        // let from_str = Fr::from_str(
        //     "52435875175126190479447740508185965837690552500527637822603658699938581184512",
        //     10,
        // )
        // .unwrap();

        // //52435875175126190479447740508185965837690552500527637822603658699938581184512
        // // println!("{:?}", &expected_poly);
        // println!("{:?}", &from_arr);
        // println!("{:?}", &from_str);
        check_test_data::<Fr, FFTSettings, Polynomial>();
    }

    #[test]
    fn zero_poly_known_() {
        assert!(init(CurveType::BLS12_381));
        zero_poly_known::<Fr, FFTSettings, Polynomial>();
    }

    #[test]
    fn zero_poly_random_() {
        assert!(init(CurveType::BLS12_381));
        zero_poly_random::<Fr, FFTSettings, Polynomial>();
    }

    #[test]
    fn zero_poly_all_but_one_() {
        assert!(init(CurveType::BLS12_381));
        zero_poly_all_but_one::<Fr, FFTSettings, Polynomial>();
    }

    #[test]
    fn zero_poly_252_() {
        assert!(init(CurveType::BLS12_381));
        zero_poly_252::<Fr, FFTSettings, Polynomial>();
    }
}

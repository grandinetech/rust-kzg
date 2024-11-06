// #[path = "./local_tests/local_poly.rs"]
// pub mod local_poly;

#[cfg(test)]
mod tests {
    use kzg_bench::tests::poly::{
        create_poly_of_length_ten, poly_div_by_zero, poly_div_fast_test, poly_div_long_test,
        poly_div_random, poly_eval_0_check, poly_eval_check, poly_eval_nil_check,
        poly_inverse_simple_0, poly_inverse_simple_1, poly_mul_direct_test, poly_mul_fft_test,
        poly_mul_random, poly_test_div,
    };
    use rust_kzg_mcl::types::fft_settings::FsFFTSettings;
    use rust_kzg_mcl::types::fr::FsFr;
    use rust_kzg_mcl::types::poly::FsPoly;

    // Local tests
    // #[test]
    // fn local_create_poly_of_length_ten_() {
    //     create_poly_of_length_ten()
    // }
    //
    // #[test]
    // fn local_poly_pad_works_rand_() {
    //     poly_pad_works_rand()
    // }
    //
    // #[test]
    // fn local_poly_eval_check_() {
    //     poly_eval_check()
    // }
    //
    // #[test]
    // fn local_poly_eval_0_check_() { poly_eval_0_check() }
    //
    // #[test]
    // fn local_poly_eval_nil_check_() {
    //     poly_eval_nil_check()
    // }
    //
    // #[test]
    // fn local_poly_inverse_simple_0_() {
    //     poly_inverse_simple_0()
    // }
    //
    // #[test]
    // fn local_poly_inverse_simple_1_() {
    //     poly_inverse_simple_1()
    // }
    //
    // #[test]
    // fn local_test_poly_div_by_zero_() {
    //     test_poly_div_by_zero()
    // }
    //
    // #[test]
    // fn local_poly_div_long_test_() {
    //     poly_div_long_test()
    // }
    //
    // #[test]
    // fn local_poly_div_fast_test_() {
    //     poly_div_fast_test()
    // }
    //
    // #[test]
    // fn local_poly_mul_direct_test_() {
    //     poly_mul_direct_test()
    // }
    //
    // #[test]
    // fn local_poly_mul_fft_test_() {
    //     poly_mul_fft_test()
    // }
    //
    // #[test]
    // fn local_poly_mul_random_() {
    //     poly_mul_random()
    // }
    //
    // #[test]
    // fn local_poly_div_random_() {
    //     poly_div_random()
    // }

    // Shared tests
    #[test]
    fn create_poly_of_length_ten_() {
        create_poly_of_length_ten::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_eval_check_() {
        poly_eval_check::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_eval_0_check_() {
        poly_eval_0_check::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_eval_nil_check_() {
        poly_eval_nil_check::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_inverse_simple_0_() {
        poly_inverse_simple_0::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_inverse_simple_1_() {
        poly_inverse_simple_1::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_test_div_() {
        poly_test_div::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_div_by_zero_() {
        poly_div_by_zero::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_mul_direct_test_() {
        poly_mul_direct_test::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_mul_fft_test_() {
        poly_mul_fft_test::<FsFr, FsPoly, FsFFTSettings>()
    }

    #[test]
    fn poly_mul_random_() {
        poly_mul_random::<FsFr, FsPoly, FsFFTSettings>()
    }

    #[test]
    fn poly_div_random_() {
        poly_div_random::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_div_long_test_() {
        poly_div_long_test::<FsFr, FsPoly>()
    }

    #[test]
    fn poly_div_fast_test_() {
        poly_div_fast_test::<FsFr, FsPoly>()
    }
}

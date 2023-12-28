// #[path = "./local_tests/local_consts.rs"]
// pub mod local_consts;

#[cfg(test)]
mod tests {
    use kzg_bench::tests::consts::{
        expand_roots_is_plausible, new_fft_settings_is_plausible, roots_of_unity_are_plausible,
        roots_of_unity_is_the_expected_size, roots_of_unity_out_of_bounds_fails,
    };
    use rust_kzg_constantine::consts::SCALE2_ROOT_OF_UNITY;
    use rust_kzg_constantine::types::fft_settings::{expand_root_of_unity, CtFFTSettings};
    use rust_kzg_constantine::types::fr::CtFr;

    // Shared tests
    #[test]
    fn roots_of_unity_is_the_expected_size_() {
        roots_of_unity_is_the_expected_size(&SCALE2_ROOT_OF_UNITY);
    }

    #[test]
    fn roots_of_unity_out_of_bounds_fails_() {
        roots_of_unity_out_of_bounds_fails::<CtFr, CtFFTSettings>();
    }

    #[test]
    fn roots_of_unity_are_plausible_() {
        roots_of_unity_are_plausible::<CtFr>(&SCALE2_ROOT_OF_UNITY);
    }

    #[test]
    fn expand_roots_is_plausible_() {
        expand_roots_is_plausible::<CtFr>(&SCALE2_ROOT_OF_UNITY, &expand_root_of_unity);
    }

    #[test]
    fn new_fft_settings_is_plausible_() {
        new_fft_settings_is_plausible::<CtFr, CtFFTSettings>();
    }

    // Local tests
    // #[test]
    // fn roots_of_unity_repeat_at_stride_() {
    //     roots_of_unity_repeat_at_stride::<CtFr, CtFFTSettings>();
    // }
}

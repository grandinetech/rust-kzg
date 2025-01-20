// #[path = "./local_tests/local_consts.rs"]
// pub mod local_consts;

#[cfg(test)]
mod tests {
    use kzg_bench::tests::consts::{
        expand_roots_is_plausible, new_fft_settings_is_plausible, roots_of_unity_are_plausible,
        roots_of_unity_is_the_expected_size, roots_of_unity_out_of_bounds_fails,
    };
    use rust_kzg_mcl::consts::SCALE2_ROOT_OF_UNITY;
    use rust_kzg_mcl::types::fft_settings::{expand_root_of_unity, MclFFTSettings};
    use rust_kzg_mcl::types::fr::MclFr;

    // Shared tests
    #[test]
    fn roots_of_unity_is_the_expected_size_() {
        roots_of_unity_is_the_expected_size(&SCALE2_ROOT_OF_UNITY);
    }

    #[test]
    fn roots_of_unity_out_of_bounds_fails_() {
        roots_of_unity_out_of_bounds_fails::<MclFr, MclFFTSettings>();
    }

    #[test]
    fn roots_of_unity_are_plausible_() {
        roots_of_unity_are_plausible::<MclFr>(&SCALE2_ROOT_OF_UNITY);
    }

    #[test]
    fn expand_roots_is_plausible_() {
        expand_roots_is_plausible::<MclFr>(&SCALE2_ROOT_OF_UNITY, &expand_root_of_unity);
    }

    #[test]
    fn new_fft_settings_is_plausible_() {
        new_fft_settings_is_plausible::<MclFr, MclFFTSettings>();
    }

    // Local tests
    // #[test]
    // fn roots_of_unity_repeat_at_stride_() {
    //     roots_of_unity_repeat_at_stride::<MclFr, MclFFTSettings>();
    // }
}

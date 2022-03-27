// #[path = "./local_tests/local_consts.rs"]
// pub mod local_consts;

#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::consts::{
        expand_roots_is_plausible, new_fft_settings_is_plausible, roots_of_unity_are_plausible,
        roots_of_unity_out_of_bounds_fails, roots_of_unity_is_the_expected_size,
    };
    use blst_from_scratch::consts::roots_of_unity_arr;
    use blst_from_scratch::types::fft_settings::{expand_root_of_unity, FsFFTSettings};
    use blst_from_scratch::types::fr::FsFr;

    // Shared tests
    #[test]
    fn roots_of_unity_is_the_expected_size_() {
        roots_of_unity_is_the_expected_size(&roots_of_unity_arr());
    }

    #[test]
    fn roots_of_unity_out_of_bounds_fails_() {
        roots_of_unity_out_of_bounds_fails::<FsFr, FsFFTSettings>();
    }

    #[test]
    fn roots_of_unity_are_plausible_() {
        roots_of_unity_are_plausible::<FsFr>(&roots_of_unity_arr());
    }

    #[test]
    fn expand_roots_is_plausible_() {
        expand_roots_is_plausible::<FsFr>(&roots_of_unity_arr(), &expand_root_of_unity);
    }

    #[test]
    fn new_fft_settings_is_plausible_() {
        new_fft_settings_is_plausible::<FsFr, FsFFTSettings>();
    }

    // Local tests
    // #[test]
    // fn roots_of_unity_repeat_at_stride_() {
    //     roots_of_unity_repeat_at_stride::<FsFr, FsFFTSettings>();
    // }
}

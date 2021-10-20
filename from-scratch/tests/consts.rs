#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::consts::{expand_roots_is_plausible, new_fft_settings_is_plausible, roots_of_unity_are_plausible, roots_of_unity_out_of_bounds_fails};
    use kzg_from_scratch::consts::{expand_root_of_unity, SCALE2_ROOT_OF_UNITY};
    use kzg_from_scratch::kzg_types::{FsFFTSettings, FsFr};

    #[test]
    fn roots_of_unity_out_of_bounds_fails_() {
        roots_of_unity_out_of_bounds_fails::<FsFr, FsFFTSettings>();
    }

    #[test]
    fn roots_of_unity_are_plausible_() {
        roots_of_unity_are_plausible::<FsFr>(&SCALE2_ROOT_OF_UNITY);
    }

    #[test]
    fn expand_roots_is_plausible_() {
        expand_roots_is_plausible::<FsFr>(&SCALE2_ROOT_OF_UNITY, &expand_root_of_unity);
    }

    #[test]
    fn new_fft_settings_is_plausible_() {
        new_fft_settings_is_plausible::<FsFr, FsFFTSettings>();
    }
}
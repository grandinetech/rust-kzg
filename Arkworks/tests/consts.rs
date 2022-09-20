#[cfg(test)]
pub mod tests {
    use arkworks::fft::SCALE2_ROOT_OF_UNITY;
    use arkworks::kzg_proofs::expand_root_of_unity;
    use arkworks::kzg_proofs::FFTSettings;
    use arkworks::kzg_types::FsFr;
    use kzg_bench::tests::consts::{
        expand_roots_is_plausible, new_fft_settings_is_plausible, roots_of_unity_are_plausible,
        roots_of_unity_is_the_expected_size, roots_of_unity_out_of_bounds_fails,
    };

    #[test]
    fn roots_of_unity_out_of_bounds_fails_() {
        roots_of_unity_out_of_bounds_fails::<FsFr, FFTSettings>();
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
        new_fft_settings_is_plausible::<FsFr, FFTSettings>();
    }

    #[test]
    fn roots_of_unity_is_the_expected_size_() {
        roots_of_unity_is_the_expected_size(&SCALE2_ROOT_OF_UNITY);
    }
}

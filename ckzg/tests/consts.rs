#[cfg(test)]
mod tests {
    use kzg::Fr;
    use kzg_bench::tests::consts::*;
    use ckzg::fftsettings::KzgFFTSettings;
    use ckzg::finite::BlstFr;
    use ckzg::consts::{expand_root_of_unity, expand_root_of_unity_mcl, SCALE2_ROOT_OF_UNITY};

    #[test]
    fn test_roots_of_unity_is_the_expected_size() {
        roots_of_unity_is_the_expected_size(&SCALE2_ROOT_OF_UNITY);
    }

    #[test]
    fn test_roots_of_unity_out_of_bounds_fails() {
        roots_of_unity_out_of_bounds_fails::<BlstFr, KzgFFTSettings>();
    }

    #[test]
    fn test_roots_of_unity_are_plausible() {
        roots_of_unity_are_plausible::<BlstFr>(&SCALE2_ROOT_OF_UNITY);
    }

    #[test]
    fn test_roots_of_unity_are_plausible_slice() {
        roots_of_unity_are_plausible_slice::<BlstFr>(&SCALE2_ROOT_OF_UNITY.iter()
            .map(|x| BlstFr::from_u64_arr(x))
            .collect::<Vec<_>>());
    }

    #[test]
    fn test_expand_roots_is_plausible() {
        expand_roots_is_plausible::<BlstFr>(&SCALE2_ROOT_OF_UNITY, &expand_root_of_unity);
    }

    #[test]
    fn test_expand_roots_is_plausible_slice() {
        expand_roots_is_plausible_slice::<BlstFr>(&SCALE2_ROOT_OF_UNITY.iter()
            .map(|x| BlstFr::from_u64_arr(x))
            .collect::<Vec<_>>(), &expand_root_of_unity_mcl);
    }

    #[test]
    fn test_new_fft_settings_is_plausible() {
        new_fft_settings_is_plausible::<BlstFr, KzgFFTSettings>();
    }
}

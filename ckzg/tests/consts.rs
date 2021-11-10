#[cfg(test)]
mod tests {
    use kzg_bench::tests::consts::{roots_of_unity_out_of_bounds_fails, roots_of_unity_are_plausible,
                                   expand_roots_is_plausible, new_fft_settings_is_plausible};
    use ckzg::fftsettings::KzgFFTSettings;
    use ckzg::finite::BlstFr;
    use ckzg::consts::{expand_root_of_unity, SCALE2_ROOT_OF_UNITY};

    #[test]
    fn test_roots_of_unity_out_of_bounds_fails() {
        roots_of_unity_out_of_bounds_fails::<BlstFr, KzgFFTSettings>();
    }

    #[test]
    fn test_roots_of_unity_are_plausible() {
        roots_of_unity_are_plausible::<BlstFr>(&SCALE2_ROOT_OF_UNITY);
    }

    #[test]
    fn test_expand_roots_is_plausible() {
        expand_roots_is_plausible::<BlstFr>(&SCALE2_ROOT_OF_UNITY, &expand_root_of_unity);
    }

    #[test]
    fn test_new_fft_settings_is_plausible() {
        new_fft_settings_is_plausible::<BlstFr, KzgFFTSettings>();
    }
}

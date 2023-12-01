#[cfg(test)]
mod tests {
    use kzg_bench::tests::consts::{
        expand_roots_is_plausible, new_fft_settings_is_plausible, roots_of_unity_are_plausible,
        roots_of_unity_is_the_expected_size, roots_of_unity_out_of_bounds_fails,
    };
    use rust_kzg_zkcrypto::consts::SCALE2_ROOT_OF_UNITY;
    use rust_kzg_zkcrypto::kzg_proofs::expand_root_of_unity;
    use rust_kzg_zkcrypto::kzg_proofs::FFTSettings;
    use rust_kzg_zkcrypto::kzg_types::ZFr;

    #[test]
    fn roots_of_unity_out_of_bounds_fails_() {
        roots_of_unity_out_of_bounds_fails::<ZFr, FFTSettings>();
    }

    #[test]
    fn roots_of_unity_are_plausible_() {
        roots_of_unity_are_plausible::<ZFr>(&SCALE2_ROOT_OF_UNITY);
    }

    #[test]
    fn expand_roots_is_plausible_() {
        expand_roots_is_plausible::<ZFr>(&SCALE2_ROOT_OF_UNITY, &expand_root_of_unity);
    }

    #[test]
    fn new_fft_settings_is_plausible_() {
        new_fft_settings_is_plausible::<ZFr, FFTSettings>();
    }

    #[test]
    fn roots_of_unity_is_the_expected_size_() {
        roots_of_unity_is_the_expected_size(&SCALE2_ROOT_OF_UNITY);
    }
}

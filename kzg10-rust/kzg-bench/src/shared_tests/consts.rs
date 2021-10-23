#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::consts::{
        expand_roots_is_plausible_slice, new_fft_settings_is_plausible,
        roots_of_unity_are_plausible_slice, roots_of_unity_out_of_bounds_fails,
    };

    use mcl_rust::data_types::fr::Fr;
    use mcl_rust::fk20_fft::{
        expand_root_of_unity, init_globals, FFTSettings, SCALE_2_ROOT_OF_UNITY,
    };
    use mcl_rust::mcl_methods::init;
    use mcl_rust::CurveType;

    #[test]
    fn roots_of_unity_out_of_bounds_fails_() {
        assert!(init(CurveType::BLS12_381));
        roots_of_unity_out_of_bounds_fails::<Fr, FFTSettings>();
    }

    #[test]
    fn roots_of_unity_are_plausible_() {
        assert!(init(CurveType::BLS12_381));
        unsafe {
            init_globals();
            roots_of_unity_are_plausible_slice::<Fr>(&SCALE_2_ROOT_OF_UNITY);
        }
    }

    #[test]
    fn expand_roots_is_plausible_() {
        assert!(init(CurveType::BLS12_381));
        unsafe {
            init_globals();
            expand_roots_is_plausible_slice::<Fr>(&SCALE_2_ROOT_OF_UNITY, &expand_root_of_unity);
        }
    }

    #[test]
    fn new_fft_settings_is_plausible_() {
        assert!(init(CurveType::BLS12_381));
        new_fft_settings_is_plausible::<Fr, FFTSettings>();
    }
}

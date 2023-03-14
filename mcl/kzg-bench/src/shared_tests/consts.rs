#[cfg(test)]
mod consts_tests {
    use kzg_bench::tests::consts::*;
    use mcl_rust::data_types::fr::Fr;
    use mcl_rust::fk20_fft::{
        expand_root_of_unity, init_globals, FFTSettings, SCALE_2_ROOT_OF_UNITY,
    };
    use mcl_rust::mcl_methods::init;
    use mcl_rust::CurveType;

    pub fn expand_root_of_unityarr(root: &Fr, _width: usize) -> Result<Vec<Fr>, String> {
        Ok(expand_root_of_unity(root))
    }

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
            let mut scale2_root_of_unity_arr: [[u64; 4]; 32] = [[0; 4]; 32];
            for i in 0..SCALE_2_ROOT_OF_UNITY.len() {
                scale2_root_of_unity_arr[i] = SCALE_2_ROOT_OF_UNITY[i].to_u64_arr();
            }
            roots_of_unity_are_plausible::<Fr>(&scale2_root_of_unity_arr);
        }
    }

    #[test]
    fn expand_roots_is_plausible_() {
        assert!(init(CurveType::BLS12_381));
        unsafe {
            init_globals();
            let mut scale2_root_of_unity_arr: [[u64; 4]; 32] = [[0; 4]; 32];
            for i in 0..SCALE_2_ROOT_OF_UNITY.len() {
                scale2_root_of_unity_arr[i] = SCALE_2_ROOT_OF_UNITY[i].to_u64_arr();
            }
            expand_roots_is_plausible::<Fr>(&scale2_root_of_unity_arr, &expand_root_of_unityarr);
        }
    }

    #[test]
    fn new_fft_settings_is_plausible_() {
        assert!(init(CurveType::BLS12_381));
        new_fft_settings_is_plausible::<Fr, FFTSettings>();
    }
}

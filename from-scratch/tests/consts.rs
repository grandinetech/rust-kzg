#[cfg(test)]
mod tests {
    use kzg_from_scratch::consts::{expand_root_of_unity, NUM_ROOTS, SCALE2_ROOT_OF_UNITY};
    use kzg_from_scratch::kzg_types::{fr_is_one, FFTSettings};
    use blst::{blst_fr_from_uint64, blst_fr_mul, blst_fr_sqr};
    use kzg::Fr;

    #[test]
    fn roots_of_unity_is_expected_size() {
        assert_eq!(NUM_ROOTS, SCALE2_ROOT_OF_UNITY.len())
    }

    #[test]
    fn roots_of_unity_out_of_bounds_fails() {
        assert!(FFTSettings::from_scale(NUM_ROOTS).is_err())
    }

    #[test]
    fn roots_of_unity_are_plausible() {
        let mut r: Fr = Fr::default();
        for i in 0..NUM_ROOTS {
            unsafe {
                blst_fr_from_uint64(&mut r, SCALE2_ROOT_OF_UNITY[i].as_ptr());
                for _j in 0..i {
                    blst_fr_sqr(&mut r, &r);
                }

                assert!(fr_is_one(&r));
            }
        }
    }

    #[test]
    fn expand_roots_is_plausible() {
        let scale: usize = 15;
        let width: usize = 1 << scale;

        let mut root: Fr = Fr::default();
        unsafe {
            blst_fr_from_uint64(&mut root, SCALE2_ROOT_OF_UNITY[scale].as_ptr());
        }

        let result = expand_root_of_unity(&root, width);
        assert!(result.is_ok());
        let expanded = result.unwrap();

        assert!(fr_is_one(&expanded[0]));
        assert!(fr_is_one(&expanded[width]));

        for i in 1..(width / 2 + 1) {
            let mut prod: Fr = Fr::default();
            unsafe {
                blst_fr_mul(&mut prod, &expanded[i], &expanded[width - i]);
            }

            assert!(fr_is_one(&prod));
        }
    }

    #[test]
    fn new_fft_settings_is_plausible() {
        let scale: usize = 21;
        let width: u64 = 1 << scale;

        let result = FFTSettings::from_scale(scale);
        assert!(result.is_ok());
        let fft_settings = result.unwrap();

        let mut prod: Fr = Fr::default();
        for i in 0..width {
            unsafe {
                blst_fr_mul(
                    &mut prod,
                    &fft_settings.expanded_roots_of_unity[i as usize],
                    &fft_settings.reverse_roots_of_unity[i as usize],
                );
            }

            assert!(fr_is_one(&prod));
        }
    }
}
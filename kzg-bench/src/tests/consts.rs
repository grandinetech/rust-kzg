use kzg::{FFTSettings, Fr};

pub fn roots_of_unity_out_of_bounds_fails<TFr: Fr, TFFTSettings: FFTSettings<TFr>>() {
    let fft_settings = TFFTSettings::new(32);
    assert!(fft_settings.is_err());
}

/// Raise each root to the power of 2 ^ i and see if it equals 1
pub fn roots_of_unity_are_plausible<TFr: Fr>(roots: &[[u64; 4]; 32]) {
    for i in 0..32 {
        let mut r = TFr::from_u64_arr(&roots[i]);
        for _j in 0..i {
            r = r.sqr();
        }

        assert!(r.is_one());
    }
}

pub fn roots_of_unity_are_plausible_slice<TFr: Fr>(roots: &[TFr]) {
    for i in 0..roots.len() {
        let mut r = roots[i].clone();
        for _j in 0..i {
            r = r.sqr();
        }

        assert!(r.is_one());
    }
}

/// Check if expanded root members follow symmetry and symmetrically multiply to produce a 1.
pub fn expand_roots_is_plausible<TFr: Fr>(
    roots: &[[u64; 4]; 32],
    expand_root_of_unity: &dyn Fn(&TFr, usize) -> Result<Vec<TFr>, String>,
) {
    let scale = 15;
    let width: usize = 1 << scale;

    let root = TFr::from_u64_arr(&roots[scale]);
    let expanded = expand_root_of_unity(&root, width).unwrap();

    assert!(expanded[0].is_one());
    assert!(expanded[width].is_one());

    // Multiply symmetrically and check if the result is 1
    for i in 0..(width / 2 + 1) {
        let prod = expanded[i].mul(&expanded[width - i]);
        assert!(prod.is_one());
    }
}

pub fn expand_roots_is_plausible_slice<TFr: Fr>(
    roots: &[TFr],
    expand_root_of_unity: &dyn Fn(&TFr) -> Vec<TFr>,
) {
    let scale = 15;
    let width: usize = 1 << scale;

    let root = &roots[scale];
    let expanded = expand_root_of_unity(&root);

    assert!(expanded[0].is_one());
    assert!(expanded[width].is_one());

    // Multiply symmetrically and check if the result is 1
    for i in 0..(width / 2 + 1) {
        let prod = expanded[i].mul(&expanded[width - i]);
        assert!(prod.is_one());
    }
}

/// Check if generated reverse roots are reversed correctly and multiply with expanded roots to result in 1.
pub fn new_fft_settings_is_plausible<TFr: Fr, TFFTSettings: FFTSettings<TFr>>() {
    let scale = 21;
    let width: usize = 1 << scale;

    let fft_settings = TFFTSettings::new(scale).unwrap();
    assert_eq!(fft_settings.get_max_width(), width);

    for i in 0..width {
        let prod = fft_settings
            .get_expanded_roots_of_unity_at(i)
            .mul(&fft_settings.get_reverse_roots_of_unity_at(i));
        assert!(prod.is_one());
    }
}
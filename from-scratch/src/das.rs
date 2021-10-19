use crate::kzg_types::FFTSettings;
use crate::utils::is_power_of_two;
use blst::{blst_fr_add, blst_fr_eucl_inverse, blst_fr_from_uint64, blst_fr_mul, blst_fr_sub};
use kzg::Fr;

// TODO: explain algo
pub fn das_fft_extension_stride(evens: &mut [Fr], stride: usize, fft_settings: &FFTSettings) {
    if evens.len() < 2 {
        return;
    } else if evens.len() == 2 {
        let mut x = Fr::default();
        let mut y = Fr::default();
        let mut y_times_root = Fr::default();

        unsafe {
            blst_fr_add(&mut x, &evens[0], &evens[1]);
            blst_fr_sub(&mut y, &evens[0], &evens[1]);
            blst_fr_mul(&mut y_times_root, &y, &fft_settings.expanded_roots_of_unity[stride]);
            blst_fr_add(&mut evens[0], &x, &y_times_root);
            blst_fr_sub(&mut evens[1], &x, &y_times_root);
        }

        return;
    }

    let half: usize = evens.len() / 2;
    for i in 0..half {
        let mut tmp1: Fr = Fr::default();
        let mut tmp2: Fr = Fr::default();

        unsafe {
            blst_fr_add(&mut tmp1, &evens[i], &evens[half + i]);
            blst_fr_sub(&mut tmp2, &evens[i], &evens[half + i]);
            blst_fr_mul(&mut evens[half + i], &tmp2, &fft_settings.reverse_roots_of_unity[i * 2 * stride]);

            evens[i] = tmp1;
        }
    }

    // Recurse
    das_fft_extension_stride(&mut evens[..half], stride * 2, &fft_settings);
    das_fft_extension_stride(&mut evens[half..], stride * 2, &fft_settings);

    for i in 0..half {
        let mut y_times_root: Fr = Fr::default();
        let x = evens[i];
        let y = evens[half + i];

        unsafe {
            blst_fr_mul(&mut y_times_root, &y, &fft_settings.expanded_roots_of_unity[(1 + 2 * i) * stride]);
            blst_fr_add(&mut evens[i], &x, &y_times_root);
            blst_fr_sub(&mut evens[half + i], &x, &y_times_root);
        }
    }
}

/// Polynomial extension for data availability sampling. Given values of even indices, produce values of odd indices.
/// FFTSettings must hold at least 2 times the roots of provided evens.
/// The resulting odd indices make the right half of the coefficients of the inverse FFT of the combined indices zero.
pub fn das_fft_extension(evens: &[Fr], fft_settings: &FFTSettings) -> Result<Vec<Fr>, String> {
    if evens.len() == 0 {
        return Err(String::from("A non-zero list ab expected"));
    } else if !is_power_of_two(evens.len()) {
        return Err(String::from("A list with power-of-two length expected"));
    } else if evens.len() * 2 > fft_settings.max_width {
        return Err(String::from("Supplied list is longer than the available max width"));
    }

    // In case more roots are provided with fft_settings, use a larger stride
    let stride = fft_settings.max_width / (evens.len() * 2);
    let mut odds = evens.to_vec();
    das_fft_extension_stride(&mut odds, stride, fft_settings);

    // TODO: explain why each odd member is multiplied by euclidean inverse of length
    unsafe {
        let mut inv_len: Fr = Fr::default();
        blst_fr_from_uint64(&mut inv_len, [odds.len() as u64, 0, 0, 0].as_ptr());
        blst_fr_eucl_inverse(&mut inv_len, &inv_len);
        for i in 0..odds.len() {
            blst_fr_mul(&mut odds[i], &odds[i], &inv_len);
        }
    }

    Ok(odds)
}


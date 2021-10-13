use crate::kzg_types::FFTSettings;
use crate::utils::is_power_of_two;
use blst::{blst_fr_add, blst_fr_eucl_inverse, blst_fr_from_uint64, blst_fr_mul, blst_fr_sub};
use kzg::Fr;

pub fn das_fft_extension_stride(evens: &mut [Fr], stride: usize, fft_settings: &FFTSettings) {
    if evens.len() < 2 {
        return;
    } else if evens.len() == 2 {
        let mut x: Fr = Fr::default();
        let mut y: Fr = Fr::default();
        let mut y_times_root: Fr = Fr::default();

        unsafe {
            blst_fr_add(&mut x, &evens[0], &evens[1]);
            blst_fr_sub(&mut y, &evens[0], &evens[1]);
            blst_fr_mul(&mut y_times_root, &y, &fft_settings.expanded_roots_of_unity[stride]);
            blst_fr_add(&mut evens[0], &x, &y_times_root);
            blst_fr_sub(&mut evens[1], &x, &y_times_root);
        }
    } else {
        let half: usize = evens.len();
        let halfhalf: usize = half / 2;

        for i in 0..halfhalf {
            let mut tmp1: Fr = Fr::default();
            let mut tmp2: Fr = Fr::default();

            unsafe {
                blst_fr_add(&mut tmp1, &evens[i], &evens[halfhalf + i]);
                blst_fr_sub(&mut tmp2, &evens[i], &evens[halfhalf + i]);
                blst_fr_mul(
                    &mut evens[halfhalf + i],
                    &tmp2,
                    &fft_settings.reverse_roots_of_unity[i * 2 * stride],
                );

                evens[i] = tmp1;
            }
        }

        // Recurse
        das_fft_extension_stride(&mut evens[..halfhalf], stride * 2, &fft_settings);
        das_fft_extension_stride(&mut evens[halfhalf..], stride * 2, &fft_settings);

        for i in 0..halfhalf {
            let mut y_times_root: Fr = Fr::default();
            let x = evens[i];
            let y = evens[halfhalf + i];

            unsafe {
                blst_fr_mul(
                    &mut y_times_root,
                    &y,
                    &fft_settings.expanded_roots_of_unity[(1 + 2 * i) * stride],
                );
                blst_fr_add(&mut evens[i], &x, &y_times_root);
                blst_fr_sub(&mut evens[halfhalf + i], &x, &y_times_root);
            }
        }
    }
}

pub fn das_fft_extension(evens: &[Fr], fft_settings: &FFTSettings) -> Result<Vec<Fr>, String> {
    if evens.len() == 0 {
        return Err(String::from("A non-zero list ab expected"));
    } else if !is_power_of_two(evens.len()) {
        return Err(String::from("A list with power-of-two length expected"));
    } else if evens.len() * 2 > fft_settings.max_width {
        return Err(String::from("Supplied list is longer than the available max width"));
    }

    let mut odds = evens.clone().to_vec();
    das_fft_extension_stride(&mut odds, fft_settings.max_width / (evens.len() * 2), fft_settings);

    let mut invlen: Fr = Fr::default();
    unsafe {
        blst_fr_from_uint64(&mut invlen, [odds.len() as u64, 0, 0, 0].as_ptr());
        blst_fr_eucl_inverse(&mut invlen, &invlen);
        for i in 0..odds.len() {
            blst_fr_mul(&mut odds[i], &odds[i], &invlen);
        }
    }

    return Ok(odds);
}


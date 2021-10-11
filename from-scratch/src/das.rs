use crate::kzg_types::FFTSettings;
use crate::utils::is_power_of_two;
use blst::{blst_fr_add, blst_fr_eucl_inverse, blst_fr_from_uint64, blst_fr_mul, blst_fr_sub};
use kzg::Fr;

pub fn das_fft_extension_stride(ab: &mut [Fr], stride: usize, fft_settings: &FFTSettings) {
    if ab.len() < 2 {
        return;
    } else if ab.len() == 2 {
        let mut x: Fr = Fr::default();
        let mut y: Fr = Fr::default();
        let mut temp: Fr = Fr::default();

        unsafe {
            blst_fr_add(&mut x, &ab[0], &ab[1]);
            blst_fr_sub(&mut y, &ab[0], &ab[1]);
            blst_fr_mul(&mut temp, &y, &fft_settings.expanded_roots_of_unity[stride]);
            blst_fr_add(&mut ab[0], &x, &temp);
            blst_fr_sub(&mut ab[1], &x, &temp);
        }
    } else {
        let half: usize = ab.len();
        let halfhalf: usize = half / 2;

        for i in 0..halfhalf {
            let mut tmp1: Fr = Fr::default();
            let mut tmp2: Fr = Fr::default();

            unsafe {
                blst_fr_add(&mut tmp1, &ab[i], &ab[halfhalf + i]);
                blst_fr_sub(&mut tmp2, &ab[i], &ab[halfhalf + i]);
                blst_fr_mul(
                    &mut ab[halfhalf + i],
                    &tmp2,
                    &fft_settings.reverse_roots_of_unity[i * 2 * stride],
                );

                ab[i] = tmp1;
            }
        }

        // Recurse
        das_fft_extension_stride(&mut ab[..halfhalf], stride * 2, &fft_settings);
        das_fft_extension_stride(&mut ab[halfhalf..], stride * 2, &fft_settings);

        for i in 0..halfhalf {
            let mut y_times_root: Fr = Fr::default();
            let x = ab[i];
            let y = ab[halfhalf + i];

            unsafe {
                blst_fr_mul(
                    &mut y_times_root,
                    &y,
                    &fft_settings.expanded_roots_of_unity[(1 + 2 * i) * stride],
                );
                blst_fr_add(&mut ab[i], &x, &y_times_root);
                blst_fr_sub(&mut ab[halfhalf + i], &x, &y_times_root);
            }
        }
    }
}

pub fn das_fft_extension(ab: &mut [Fr], fft_settings: &FFTSettings) -> Result<(), String> {
    if ab.len() == 0 {
        return Err(String::from("A non-zero list ab expected"));
    } else if !is_power_of_two(ab.len()) {
        return Err(String::from("A list with power-of-two length expected"));
    } else if ab.len() * 2 > fft_settings.max_width {
        return Err(String::from(
            "Supplied list is longer than the available max width",
        ));
    }

    das_fft_extension_stride(ab, fft_settings.max_width / (ab.len() * 2), fft_settings);

    let mut invlen: Fr = Fr::default();
    unsafe {
        blst_fr_from_uint64(&mut invlen, [ab.len() as u64, 0, 0, 0].as_ptr());
        blst_fr_eucl_inverse(&mut invlen, &invlen);
        for i in 0..ab.len() {
            blst_fr_mul(&mut ab[i], &ab[i], &invlen);
        }
    }

    return Ok(());
}

#[cfg(test)]
mod tests {
    use crate::das::das_fft_extension;
    use crate::kzg_types::{create_fr_rand, fr_are_equal, FFTSettings, fr_is_zero};
    use blst::blst_fr_from_uint64;
    use kzg::Fr;
    use crate::fft_fr::fft_fr;

    #[test]
    fn das_extension_test_known() {
        let expected_u: [[u64; 4]; 8] = [
            [0xa0c43757db972d7d, 0x79d15a1e0677962c, 0xf678865c0c95fa6a, 0x4e85fd4814f96825, ],
            [0xad9f844939f2705d, 0x319e440c9f3b0325, 0x4cbd29a60e160a28, 0x665961d85d90c4c0, ],
            [0x5f3ac8a72468d28b, 0xede949e28383c5d2, 0xaf6f84dd8708d8c9, 0x2567aa0b14a41521, ],
            [0x25abe312b96aadad, 0x4abf043f091ff417, 0x43824b53e09536db, 0x195dbe06a28ca227, ],
            [0x5f3ac8a72468d28b, 0xede949e28383c5d2, 0xaf6f84dd8708d8c9, 0x2567aa0b14a41521, ],
            [0xad9f844939f2705d, 0x319e440c9f3b0325, 0x4cbd29a60e160a28, 0x665961d85d90c4c0, ],
            [0xa0c43757db972d7d, 0x79d15a1e0677962c, 0xf678865c0c95fa6a, 0x4e85fd4814f96825, ],
            [0x7f171458d2b071a9, 0xd185bbb2a46cbd9b, 0xa41aab0d02886e80, 0x01cacceef58ccee9, ],
        ];

        let result = FFTSettings::from_scale(4);
        assert!(result.is_ok());

        let fft_settings = result.unwrap();
        let half = fft_settings.max_width / 2;

        let mut data = vec![Fr::default(); half];
        for i in 0..half {
            unsafe {
                blst_fr_from_uint64(&mut data[i], [i as u64, 0, 0, 0].as_ptr());
            }
        }

        let result = das_fft_extension(&mut data, &fft_settings);
        assert!(result.is_ok());

        for i in 0..expected_u.len() {
            let mut expected: Fr = Fr::default();
            unsafe {
                blst_fr_from_uint64(&mut expected, expected_u[i].as_ptr());
            }

            assert!(fr_are_equal(&expected, &data[i]));
        }
    }

    #[test]
    fn das_extension_test_random() {
        let max_scale: usize = 15;
        let result = FFTSettings::from_scale(max_scale);
        assert!(result.is_ok());

        let fft_settings = result.unwrap();
        for scale in 1..(max_scale + 1) {
            let width: usize = 1 << scale;
            assert!(width <= fft_settings.max_width);

            let mut even_data = vec![Fr::default(); width / 2];
            let mut odd_data = vec![Fr::default(); width / 2];
            let mut data = vec![Fr::default(); width];
            let mut coeffs = vec![Fr::default(); width];

            for _rep in 0..4 {
                // Initialize even data and duplicate it in even data
                for i in 0..(width / 2) {
                    even_data[i] = create_fr_rand();
                    odd_data[i] = even_data[i].clone();
                }

                // Extend the even data to odd data
                let result = das_fft_extension(&mut odd_data, &fft_settings);
                assert!(result.is_ok());

                for i in (0..width).step_by(2) {
                    data[i] = even_data[i / 2];
                    data[i + 1] = odd_data[i / 2];
                }

                let result = fft_fr(&mut coeffs, &data, true, &fft_settings);
                assert!(result.is_ok());

                for i in (width / 2)..(width) {
                    assert!(fr_is_zero(&coeffs[i]));
                }
            }
        }
    }
}

use kzg::Fr;
use blst::{blst_fr_mul, blst_fr_sub, blst_fr_add, blst_fr_from_uint64, blst_fr_inverse};
use crate::kzg_types::FFTSettings;
use crate::utils::is_power_of_two;

pub fn fft_fr_fast(ret: &mut [Fr], data: &[Fr], stride: usize, roots: &[Fr], roots_stride: usize) {
    let half: usize = ret.len() / 2;
    if half > 0 {
        fft_fr_fast(&mut ret[..half], data, stride * 2, roots, roots_stride * 2);
        fft_fr_fast(&mut ret[half..], &data[stride..], stride * 2, roots, roots_stride * 2);
        for i in 0..half {
            let mut y_times_root: Fr = Fr::default();
            unsafe {
                blst_fr_mul(&mut y_times_root, &ret[i + half], &roots[i * roots_stride]);
                blst_fr_sub(&mut ret[i + half], &ret[i], &y_times_root);
                blst_fr_add(&mut ret[i], &mut ret[i], &y_times_root);
            }
        }
    } else {
        ret[0] = data[0].clone();
    }
}

pub fn fft_fr(ret: &mut [Fr], data: &[Fr], inverse: bool, fft_settings: &FFTSettings) -> Result<(), String> {
    if data.len() > fft_settings.max_width {
        return Err(String::from(
            "Supplied list is longer than the available max width",
        ));
    } else if !is_power_of_two(data.len()) {
        return Err(String::from("A list with power-of-two length expected"));
    }

    let stride: usize = fft_settings.max_width / data.len();

    if inverse {
        let mut inv_len: Fr = Fr::default();
        unsafe {
            blst_fr_from_uint64(&mut inv_len, [data.len() as u64, 0, 0, 0].as_ptr());
            blst_fr_inverse(&mut inv_len, &inv_len);
        }
        fft_fr_fast(ret, data, 1, &fft_settings.reverse_roots_of_unity, stride);
        for i in 0..data.len() {
            unsafe {
                blst_fr_mul(&mut ret[i], &ret[i], &inv_len);
            }
        }
    } else {
        fft_fr_fast(ret, data, 1, &fft_settings.expanded_roots_of_unity, stride);
    }

    return Ok(());
}

pub fn fft_fr_slow(ret: &mut [Fr], data: &[Fr], stride: usize, roots: &[Fr], roots_stride: usize) {
    let mut last: Fr = Fr::default();
    let mut v: Fr = Fr::default();
    for i in 0..data.len() {
        unsafe {
            blst_fr_mul(&mut last, &data[0], &roots[0]);
        }

        for j in 1..data.len() {
            let jv = data[j * stride];
            let r = roots[((i * j) % data.len()) * roots_stride];
            unsafe {
                blst_fr_mul(&mut v, &jv, &r);
                blst_fr_add(&mut last, &last, &v);
            }
        }

        ret[i] = last;
    }
}

#[cfg(test)]
mod tests {
    use crate::kzg_types::{FFTSettings, fr_are_equal};
    use blst::{blst_fr_from_uint64, blst_fr_from};
    use kzg::Fr;
    use crate::fft_fr::{fft_fr_slow, fft_fr_fast, fft_fr};

    #[test]
    fn compare_sft_fft() {
        let size: usize = 12;

        let result = FFTSettings::from_scale(size);
        assert!(result.is_ok());
        let fft_settings = result.unwrap();

        let mut data = vec![Fr::default(); fft_settings.max_width];
        for i in 0..fft_settings.max_width {
            unsafe {
                blst_fr_from_uint64(&mut data[i], [i as u64, 0, 0, 0].as_ptr());
            }
        }

        let mut out0 = vec![Fr::default(); fft_settings.max_width];
        let mut out1 = vec![Fr::default(); fft_settings.max_width];

        // Compare fast and slow FFT approach
        fft_fr_slow(&mut out0, &data, 1, &fft_settings.expanded_roots_of_unity, 1);
        fft_fr_fast(&mut out1, &data, 1, &fft_settings.expanded_roots_of_unity, 1);

        for i in 0..fft_settings.max_width {
            assert!(fr_are_equal(&out0[i], &out1[i]));
        }
    }

    #[test]
    fn roundtrip_fft() {
        let size: usize = 12;

        let result = FFTSettings::from_scale(size);
        assert!(result.is_ok());
        let fft_settings = result.unwrap();

        let mut coeffs = vec![Fr::default(); fft_settings.max_width];
        let mut data = vec![Fr::default(); fft_settings.max_width];
        for i in 0..fft_settings.max_width {
            unsafe {
                blst_fr_from_uint64(&mut data[i], [i as u64, 0, 0, 0].as_ptr());
            }
        }

        // Forward and inverse FFT
        let result = fft_fr(&mut coeffs, &data, false, &fft_settings);
        assert!(result.is_ok());

        let result = fft_fr(&mut data, &coeffs, true, &fft_settings);
        assert!(result.is_ok());

        for i in 0..fft_settings.max_width {
            let mut temp: Fr = Fr::default();
            unsafe {
                blst_fr_from_uint64(&mut temp, [i as u64, 0, 0, 0].as_ptr());
            }

            assert!(fr_are_equal(&temp, &data[i]));
        }
    }

    #[test]
    fn inverse_fft() {
        let inv_fft_expected: [[u64; 4]; 16] =
            [
                [0x7fffffff80000008, 0xa9ded2017fff2dff, 0x199cec0404d0ec02, 0x39f6d3a994cebea4],
                [0xef296e7ffb8ca216, 0xd5b902cbcef9c1b6, 0xf06dfe5c7fca260d, 0x13993b7d05187205],
                [0xe930fdda2306c7d4, 0x40e02aff48e2b16b, 0x83a712d1dd818c8f, 0x5dbc603bc53c7a3a],
                [0xf9925986d0d25e90, 0xcdf85d0a339d7782, 0xee7a9a5f0410e423, 0x2e0d216170831056],
                [0x80007fff80000000, 0x1fe05202bb00adff, 0x6045d26b3fd26e6b, 0x39f6d3a994cebea4],
                [0x27325dd08ac4cee9, 0xcbb94f168ddacca9, 0x6843be68485784b1, 0x5a6faf9039451673],
                [0xe92ffdda2306c7d4, 0x54dd2afcd2dfb16b, 0xf6554603677e87be, 0x5dbc603bc53c7a39],
                [0x1cc772c9b57f126f, 0xfb73f4d33d3116dd, 0x4f9388c8d80abcf9, 0x3ffbc9abcdda7821],
                [0x7fffffff80000000, 0xa9ded2017fff2dff, 0x199cec0404d0ec02, 0x39f6d3a994cebea4],
                [0xe3388d354a80ed91, 0x5849af2fc2cd4521, 0xe3a64f3f31971b0b, 0x33f1dda75bc30526],
                [0x16d00224dcf9382c, 0xfee079062d1eaa93, 0x3ce49204a2235046, 0x163147176461030e],
                [0xd8cda22e753b3117, 0x880454ec72238f55, 0xcaf6199fc14a5353, 0x197df7c2f05866d4],
                [0x7fff7fff80000000, 0x33dd520044fdadff, 0xd2f4059cc9cf699a, 0x39f6d3a994cebea3],
                [0x066da6782f2da170, 0x85c546f8cc60e47c, 0x44bf3da90590f3e1, 0x45e085f1b91a6cf1],
                [0x16cf0224dcf9382c, 0x12dd7903b71baa93, 0xaf92c5362c204b76, 0x163147176461030d],
                [0x10d6917f04735dea, 0x7e04a13731049a48, 0x42cbd9ab89d7b1f7, 0x60546bd624850b42]
            ];

        let result = FFTSettings::from_scale(4);
        assert!(result.is_ok());
        let fft_settings = result.unwrap();

        let mut data = vec![Fr::default(); fft_settings.max_width];
        let mut out = vec![Fr::default(); fft_settings.max_width];
        for i in 0..fft_settings.max_width {
            unsafe {
                blst_fr_from_uint64(&mut data[i], [i as u64, 0, 0, 0].as_ptr());
            }
        }

        let result = fft_fr(&mut out, &data, true, &fft_settings);
        assert!(result.is_ok());

        assert_eq!(inv_fft_expected.len(), fft_settings.max_width);
        for i in 0..inv_fft_expected.len() {
            let mut expected: Fr = Fr::default();
            unsafe {
                blst_fr_from_uint64(&mut expected, inv_fft_expected[i].as_ptr());
            }
            assert!(fr_are_equal(&expected, &out[i]));
        }
    }

    #[test]
    fn stride_fft() {
        let size1: usize = 9;
        let size2: usize = 12;

        let width: usize = 1 << size1;

        let result = FFTSettings::from_scale(size1);
        assert!(result.is_ok());
        let fft_settings1 = result.unwrap();

        let result = FFTSettings::from_scale(size2);
        assert!(result.is_ok());
        let fft_settings2 = result.unwrap();

        let mut data = vec![Fr::default(); width];
        let mut coeffs1 = vec![Fr::default(); width];
        let mut coeffs2 = vec![Fr::default(); width];
        for i in 0..width {
            unsafe {
                blst_fr_from_uint64(&mut data[i], [i as u64, 0, 0, 0].as_ptr());
            }
        }

        // Forward and inverse FFT
        let result = fft_fr(&mut coeffs1, &data, false, &fft_settings1);
        assert!(result.is_ok());

        let result = fft_fr(&mut coeffs2, &data, false, &fft_settings2);
        assert!(result.is_ok());

        for i in 0..width {
            assert!(fr_are_equal(&coeffs1[i], &coeffs2[i]));
        }
    }
}

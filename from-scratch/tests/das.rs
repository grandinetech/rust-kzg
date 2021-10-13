#[cfg(test)]
mod tests {
    use blst::blst_fr_from_uint64;
    use kzg::Fr;
    use kzg_from_scratch::das::das_fft_extension;
    use kzg_from_scratch::fft_fr::fft_fr;
    use kzg_from_scratch::kzg_types::{create_fr_rand, FFTSettings, fr_are_equal, fr_is_zero};

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

        let mut evens = vec![Fr::default(); half];
        for i in 0..half {
            unsafe {
                blst_fr_from_uint64(&mut evens[i], [i as u64, 0, 0, 0].as_ptr());
            }
        }

        let result = das_fft_extension(&mut evens, &fft_settings);
        assert!(result.is_ok());
        let odds = result.unwrap();

        for i in 0..expected_u.len() {
            let mut expected: Fr = Fr::default();
            unsafe {
                blst_fr_from_uint64(&mut expected, expected_u[i].as_ptr());
            }

            assert!(fr_are_equal(&expected, &odds[i]));
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

            let mut evens = vec![Fr::default(); width / 2];
            let mut data = vec![Fr::default(); width];

            for _rep in 0..4 {
                // Initialize even data and duplicate it in even data
                for i in 0..(width / 2) {
                    evens[i] = create_fr_rand();
                }

                // Extend the even data to odd data
                let result = das_fft_extension(&evens, &fft_settings);
                assert!(result.is_ok());
                let odds = result.unwrap();

                for i in (0..width).step_by(2) {
                    data[i] = evens[i / 2];
                    data[i + 1] = odds[i / 2];
                }

                let result = fft_fr(&data, true, &fft_settings);
                assert!(result.is_ok());
                let coeffs = result.unwrap();

                for i in (width / 2)..(width) {
                    assert!(fr_is_zero(&coeffs[i]));
                }
            }
        }
    }
}

use kzg::{IFFTSettings, IFr};

/// Check if DAS FFT creates odds that match precomputed values
pub fn das_extension_test_known<TFr: IFr, TFFTSettings: IFFTSettings<TFr>>(
    das_fft_extension: &dyn Fn(&[TFr], &TFFTSettings) -> Result<Vec<TFr>, String>
) {
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

    let fft_settings = TFFTSettings::new(4).unwrap();

    let mut evens = Vec::new();
    for i in 0..(fft_settings.get_max_width() / 2) {
        let temp = TFr::from_u64(i as u64);
        evens.push(temp);
    }

    let odds = das_fft_extension(&mut evens, &fft_settings).unwrap();

    for i in 0..expected_u.len() {
        let expected = TFr::from_u64_arr(&expected_u[i]);
        assert!(expected.equals(&odds[i]));
    }
}

/// Check that DAS extension produces correct odds.
/// Verify this by checking that the second half of the inverse FFT coefficients of odd-even interpolated vector results in zeros.

pub fn das_extension_test_random<TFr: IFr, TFFTSettings: IFFTSettings<TFr>>(
    das_fft_extension: &dyn Fn(&[TFr], &TFFTSettings) -> Result<Vec<TFr>, String>,
    fft_fr: &dyn Fn(&[TFr], bool, &TFFTSettings) -> Result<Vec<TFr>, String>,
) {
    let max_scale: usize = 15;

    let fft_settings = TFFTSettings::new(max_scale).unwrap();

    for scale in 1..(max_scale + 1) {
        let width: usize = 1 << scale;
        assert!(width <= fft_settings.get_max_width());

        for _rep in 0..4 {
            let mut evens = Vec::new();
            for _i in 0..(width / 2) {
                evens.push(TFr::rand());
            }

            let odds = das_fft_extension(&evens, &fft_settings).unwrap();

            let mut data = Vec::new();
            for i in (0..width).step_by(2) {
                data.push(evens[i / 2].clone());
                data.push(odds[i / 2].clone());
            }

            let coeffs = fft_fr(&data, true, &fft_settings).unwrap();

            for i in (width / 2)..(width) {
                assert!(coeffs[i].is_zero());
            }
        }
    }
}

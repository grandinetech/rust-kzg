use kzg::{FFTFr, FFTSettings, Fr};

/// Check that both FFT implementations produce the same results
#[allow(clippy::type_complexity)]
pub fn compare_sft_fft<TFr: Fr, TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>>(
    fft_fr_slow: &dyn Fn(&mut [TFr], &[TFr], usize, &[TFr], usize),
    fft_fr_fast: &dyn Fn(&mut [TFr], &[TFr], usize, &[TFr], usize),
) {
    let size: usize = 12;

    let fft_settings = TFFTSettings::new(size).unwrap();

    let data = (0..fft_settings.get_max_width())
        .map(|i| TFr::from_u64_arr(&[i as u64, 0, 0, 0]))
        .collect::<Vec<_>>();

    let mut out0 = vec![TFr::default(); fft_settings.get_max_width()];
    let mut out1 = vec![TFr::default(); fft_settings.get_max_width()];

    // Compare fast and slow FFT approach
    fft_fr_slow(
        &mut out0,
        &data,
        1,
        fft_settings.get_roots_of_unity(),
        1,
    );
    fft_fr_fast(
        &mut out1,
        &data,
        1,
        fft_settings.get_roots_of_unity(),
        1,
    );

    for i in 0..fft_settings.get_max_width() {
        assert!(out0[i].equals(&out1[i]));
    }
}

/// Check that computing FFT and inverse FFT results in the starting data
pub fn roundtrip_fft<TFr: Fr, TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>>() {
    let size: usize = 12;

    let fft_settings = TFFTSettings::new(size).unwrap();

    let starting_data = (0..fft_settings.get_max_width())
        .map(|i| TFr::from_u64(i as u64))
        .collect::<Vec<_>>();

    // Forward and inverse FFT
    let forward_result = fft_settings.fft_fr(&starting_data, false).unwrap();
    let inverse_result = fft_settings.fft_fr(&forward_result, true).unwrap();

    for (i, data) in starting_data.iter().enumerate() {
        assert!(data.equals(&inverse_result[i]));
    }
}

/// Check the inverse FFT operation on precomputed values
pub fn inverse_fft<TFr: Fr, TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>>() {
    #[rustfmt::skip]
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

    let fft_settings = TFFTSettings::new(4).unwrap();

    let data = (0..fft_settings.get_max_width())
        .map(|i| TFr::from_u64(i as u64))
        .collect::<Vec<_>>();

    let forward_result = fft_settings.fft_fr(&data, true).unwrap();

    assert_eq!(inv_fft_expected.len(), fft_settings.get_max_width());
    for i in 0..inv_fft_expected.len() {
        let expected = TFr::from_u64_arr(&inv_fft_expected[i]);
        assert!(expected.equals(&forward_result[i]));
    }
}

/// Check that stride is normalized when roots of different precision are used
pub fn stride_fft<TFr: Fr, TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>>() {
    let size1: usize = 9;
    let size2: usize = 12;

    let width: usize = 1 << size1;

    let fft_settings1 = TFFTSettings::new(size1).unwrap();
    let fft_settings2 = TFFTSettings::new(size2).unwrap();

    let data = (0..width)
        .map(|i| TFr::from_u64(i as u64))
        .collect::<Vec<_>>();

    let result1 = fft_settings1.fft_fr(&data, false).unwrap();
    let result2 = fft_settings2.fft_fr(&data, false).unwrap();

    for (i, r1) in result1.iter().enumerate() {
        assert!(r1.equals(&result2[i]));
    }
}

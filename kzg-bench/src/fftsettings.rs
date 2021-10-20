#[cfg(test)]
mod tests {
    use kzg::{FFTSettings, Fr};

    #[test]
    fn test_fft_settings_alloc() {
        let mut settings = match FFTSettings::new(16) {
            Ok(s) => s,
            Err(_) => FFTSettings::default()
        };
        assert_eq!(settings.max_width, 2 << 16 - 1);
        settings.destroy();
    }

    #[test]
    fn roundtrip_fft_fr() {
        let size: u32 = 12;
        let mut fs = match FFTSettings::new(size) {
            Ok(s) => s,
            Err(_) => FFTSettings::default()
        };
        assert_eq!(fs.max_width, 2 << size - 1);
        let mut data = vec![Fr::default(); fs.max_width];
        for i in 0..fs.max_width {
            data[i] = Fr::from_u64(i as u64);
        }
        let mut coeffs = fs.fft_fr(&mut data, false);
        assert_eq!(coeffs.len(), 2 << size - 1);
        data = fs.fft_fr(&mut coeffs, true);
        assert_eq!(data.len(), 2 << size - 1);
        // Verify that the result is still ascending values of i
        for i in 0..fs.max_width {
            let temp = Fr::from_u64(i as u64);
            assert_eq!(Fr::is_equal(temp, data[i]), true);
        }
        fs.destroy();
    }

    #[test]
    fn roundtrip_fft_g1() {
        let size: u32 = 10;
        let mut fs = match FFTSettings::new(size) {
            Ok(s) => s,
            Err(_) => FFTSettings::default()
        };
        assert_eq!(fs.max_width, 2 << size - 1);
        // make_data
        let expected = FFTSettings::make_data(fs.max_width);
        let mut data = FFTSettings::make_data(fs.max_width);
        // Forward and reverse FFT
        let mut coeffs = fs.fft_g1(&mut data, false);
        assert_eq!(coeffs.len(), 2 << size - 1);
        data = fs.fft_g1(&mut coeffs, true);
        assert_eq!(data.len(), 2 << size - 1);
        // Verify that the result is still ascending values of i
        for i in 0..fs.max_width {
            assert_eq!(FFTSettings::g1_equal(&expected[i], &data[i]), true);
        }
        fs.destroy();
    }
}
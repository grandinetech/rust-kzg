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
    fn roundtrip_fft() {
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
        let mut coeffs = fs.fft(&mut data, false);
        assert_eq!(coeffs.len(), 2 << size - 1);
        data = fs.fft(&mut coeffs, true);
        assert_eq!(data.len(), 2 << size - 1);
        // Verify that the result is still ascending values of i
        for i in 0..fs.max_width {
            let temp = Fr::from_u64(i as u64);
            assert_eq!(Fr::is_equal(temp, data[i]), true);
        }
        fs.destroy();
    }
}

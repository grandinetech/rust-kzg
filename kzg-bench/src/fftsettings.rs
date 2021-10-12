#[cfg(test)]
mod tests {
    use kzg::FFTSettings;

    #[test]
    fn test_fft_settings_alloc() {
        let settings = FFTSettings::new(16);
        assert!(settings.is_ok());
        FFTSettings::free(&mut settings.unwrap());
    }
}

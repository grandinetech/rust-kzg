#[cfg(test)]
mod tests {
    use kzg::FFTSettings;

    #[test]
    fn test_fft_settings_alloc() {
        assert!(FFTSettings::new(16).is_ok());
        // no free needed here, allocation on stack
    }
}

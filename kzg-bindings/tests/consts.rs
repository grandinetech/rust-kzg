#[cfg(test)]
mod tests {
    use kzg_bench::tests::consts::{roots_of_unity_out_of_bounds_fails, new_fft_settings_is_plausible};
    use kzg_bindings::fftsettings::KzgFFTSettings;
    use kzg_bindings::finite::BlstFr;

    #[test]
    fn test_roots_of_unity_out_of_bounds_fails() {
        roots_of_unity_out_of_bounds_fails::<BlstFr, KzgFFTSettings>();
    }

    #[test]
    fn test_new_fft_settings_is_plausible() {
        new_fft_settings_is_plausible::<BlstFr, KzgFFTSettings>();
    }
}

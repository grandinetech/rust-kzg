#[cfg(test)]
mod tests {
    use kzg::fftsettings::{FFTSettings, ckzg_new_fft_settings};
    use kzg::common::KzgRet;
    use kzg::finite::BlstFr;

    #[test]
    fn test_fft_settings_alloc() {
        let root_of_unity_poly: [u64; 4] = [0, 0, 0, 0];
        let root_of_unity = &mut BlstFr { l: root_of_unity_poly };
        let expanded_roots_of_unity_poly: [u64; 4] = [0, 0, 0, 0];
        let expanded_roots_of_unity = &mut BlstFr { l: expanded_roots_of_unity_poly };
        let reverse_roots_of_unity_poly: [u64; 4] = [0, 0, 0, 0];
        let reverse_roots_of_unity = &mut BlstFr { l: reverse_roots_of_unity_poly };
        let settings = &mut FFTSettings {
            max_width: 16,
            root_of_unity: root_of_unity,
            expanded_roots_of_unity: expanded_roots_of_unity,
            reverse_roots_of_unity: reverse_roots_of_unity
        };

        assert_eq!(ckzg_new_fft_settings(settings, 16), KzgRet::KzgOk);
        // no free needed here, allocation on stack
    }

}

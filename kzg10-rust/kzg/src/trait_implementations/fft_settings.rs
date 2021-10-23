use crate::data_types::fr::Fr;
use crate::fk20_fft::{FFTSettings, SCALE_2_ROOT_OF_UNITY};
use kzg::FFTSettings as CommonFFTSettings;

impl CommonFFTSettings<Fr> for FFTSettings {
    fn new(scale: usize) -> Result<FFTSettings, String> {
        unsafe {
            if scale >= SCALE_2_ROOT_OF_UNITY.len() {
                return Err(String::from(
                    "Scale is expected to be within root of unity matrix row size",
                ));
            }
        }
        Ok(FFTSettings::new(scale as u8))
    }

    fn get_max_width(&self) -> usize {
        self.max_width
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> Fr {
        self.exp_roots_of_unity[i]
    }

    fn get_expanded_roots_of_unity(&self) -> &[Fr] {
        &self.exp_roots_of_unity
    }

    fn get_reverse_roots_of_unity_at(&self, i: usize) -> Fr {
        self.exp_roots_of_unity_rev[i]
    }

    fn get_reversed_roots_of_unity(&self) -> &[Fr] {
        &self.exp_roots_of_unity_rev
    }

    fn destroy(&mut self) {}
}

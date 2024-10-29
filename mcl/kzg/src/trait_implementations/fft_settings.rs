use crate::data_types::fr::Fr;
use crate::fk20_fft::{FFTSettings, SCALE_2_ROOT_OF_UNITY_PR7_STRINGS};
use kzg::FFTSettings as CommonFFTSettings;

impl CommonFFTSettings<Fr> for FFTSettings {
    fn new(scale: usize) -> Result<FFTSettings, String> {
        //currently alawys use PR 7 for shared tests
        FFTSettings::new_custom_primitive_roots(scale as u8, SCALE_2_ROOT_OF_UNITY_PR7_STRINGS)
    }

    fn get_max_width(&self) -> usize {
        self.max_width
    }

    // fn get_expanded_roots_of_unity_at(&self, i: usize) -> Fr {
    //     self.expanded_roots_of_unity[i]
    // }

    // fn get_expanded_roots_of_unity(&self) -> &[Fr] {
    //     &self.expanded_roots_of_unity
    // }

    fn get_reverse_roots_of_unity_at(&self, i: usize) -> Fr {
        self.reverse_roots_of_unity[i]
    }

    fn get_reversed_roots_of_unity(&self) -> &[Fr] {
        &self.reverse_roots_of_unity
    }

    fn get_roots_of_unity_at(&self, i: usize) -> Fr {
        self.roots_of_unity[i]
    }

    fn get_roots_of_unity(&self) -> &[Fr] {
        &self.roots_of_unity
    }

    fn get_brp_roots_of_unity(&self) -> &[Fr] {
        &self.brp_roots_of_unity
    }
    fn get_brp_roots_of_unity_at(&self, i: usize) -> Fr {
        self.brp_roots_of_unity[i]
    }
}

use crate::data_types::g1::G1;
use crate::fk20_fft::FFTSettings;
use kzg::FFTG1 as FftG1;

impl FftG1<G1> for FFTSettings {
    fn fft_g1(&self, data: &[G1], inverse: bool) -> Result<Vec<G1>, String> {
        if inverse {
            FFTSettings::fft_g1_inv(self, &data.to_vec())
        } else {
            FFTSettings::fft_g1(self, &data.to_vec())
        }
    }
}

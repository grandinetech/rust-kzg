use crate::data_types::fr::Fr;
use crate::fk20_fft::{FFTSettings};
use kzg::FFTFr;

impl FFTFr<Fr> for FFTSettings {
    fn fft_fr(&self, data: &[Fr], inverse: bool) -> Result<Vec<Fr>, String>{
        FFTSettings::fft(self, data, inverse)
    }
}

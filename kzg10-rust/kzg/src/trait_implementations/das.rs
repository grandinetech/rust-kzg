use crate::data_types::fr::Fr;
use crate::fk20_fft::FFTSettings;
use kzg::DAS as Das;

impl Das<Fr> for FFTSettings {
    fn das_fft_extension(&self, evens: &[Fr]) -> Result<Vec<Fr>, String> {
        Ok(FFTSettings::das_fft_extension_from_slice(self, evens))
    }
}

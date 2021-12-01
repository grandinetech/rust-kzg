use crate::data_types::fr::Fr;
use crate::fk20_fft::FFTSettings;
use kzg::DAS as Das;

impl Das<Fr> for FFTSettings {
    fn das_fft_extension(&self, evens: &[Fr]) -> Result<Vec<Fr>, String> {
        let mut values = evens.to_vec();
        FFTSettings::das_fft_extension(self, &mut values).unwrap();
        Ok(values)
    }
}

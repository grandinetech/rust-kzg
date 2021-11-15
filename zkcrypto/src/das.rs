use crate::fftsettings::ZkFFTSettings;
use crate::zkfr::blsScalar;
use kzg::{Fr, DAS};

impl DAS<blsScalar> for ZkFFTSettings {
    fn das_fft_extension(&self, val: &[blsScalar]) -> Result<Vec<blsScalar>, String> {
        assert!(val.len() > 0);
        assert!(val.len().is_power_of_two());
        assert!(val.len() * 2 <= self.max_width);

        let mut vals = val.to_vec();
        let stride = self.max_width / (vals.len() * 2);

        self.das_fft_extension_stride(&mut vals, stride);

        let invlen = blsScalar::from_u64(vals.len() as u64);
        let invlen = invlen.inverse();

        for i in 0..vals.len(){
            vals[i] = vals[i].mul(&invlen);
        }

        Ok(vals)
    }
}
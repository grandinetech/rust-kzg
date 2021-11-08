use crate::kzg_proofs::FFTSettings;
use crate::kzg_types::FsFr as BlstFr;
use kzg::{Fr, DAS};

impl FFTSettings {
    pub fn das_fft_extension_stride(&self, ab: &mut [BlstFr], stride: usize) {
        if ab.len() < 2 {
            return;
        } else if ab.len() == 2 {
            let x = ab[0].add(&ab[1]);
            let y = ab[0].sub(&ab[1]);
            let tmp = y.mul(&self.expanded_roots_of_unity[stride]);

            ab[0] = x.add(&tmp);
            ab[1] = x.sub(&tmp);

            return;
        }else {
            let half = ab.len();
            let halfhalf = half / 2;

            for i in 0..halfhalf{
                let tmp1 = ab[i].add(&ab[halfhalf+i]);
                let tmp2 = ab[i].sub(&ab[halfhalf+i]);
                ab[halfhalf+i] = tmp2.mul(&self.reverse_roots_of_unity[i * 2 * stride]);
                ab[i] = tmp1;
            }

            self.das_fft_extension_stride(&mut ab[..halfhalf], stride * 2);
            self.das_fft_extension_stride(&mut ab[halfhalf..], stride * 2);

            for i in 0..halfhalf{
                let x = ab[i];
                let y = ab[halfhalf+i];
                let y_times_root = y.mul(&self.expanded_roots_of_unity[(1 + 2 * i) * stride]);
                ab[i] = x.add(&y_times_root);
                ab[halfhalf+i] = x.sub(&y_times_root);
            }
        }
    }
}

impl DAS<BlstFr> for FFTSettings {
    fn das_fft_extension(&self, vals: &[BlstFr]) -> Result<Vec<BlstFr>, String> {
        assert!(vals.len() > 0);
        assert!(vals.len().is_power_of_two());
        assert!(vals.len() * 2 <= self.max_width);

        let mut vals = vals.to_vec();
        let stride = self.max_width / (vals.len() * 2);

        self.das_fft_extension_stride(&mut vals, stride);

        let invlen = BlstFr::from_u64(vals.len() as u64);
        let invlen = invlen.inverse();

        for i in 0..vals.len(){
            vals[i] = vals[i].mul(&invlen);
        }

        Ok(vals)
    }
}

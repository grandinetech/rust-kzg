use crate::kzg_proofs::FFTSettings;
use crate::kzg_types::ArkFr as BlstFr;
use kzg::{Fr, DAS};
use std::cmp::Ordering;

impl FFTSettings {
    fn das_fft_extension_stride(&self, ab: &mut [BlstFr], stride: usize) {
        match ab.len().cmp(&2_usize) {
            Ordering::Less => {}
            Ordering::Greater => {
                let half = ab.len();
                let halfhalf = half / 2;

                for i in 0..halfhalf {
                    let tmp1 = ab[i].add(&ab[halfhalf + i]);
                    let tmp2 = ab[i].sub(&ab[halfhalf + i]);
                    ab[halfhalf + i] = tmp2.mul(&self.reverse_roots_of_unity[i * 2 * stride]);
                    ab[i] = tmp1;
                }

                #[cfg(feature = "parallel")]
                {
                    if ab.len() > 32 {
                        let (lo, hi) = ab.split_at_mut(halfhalf);
                        rayon::join(
                            || self.das_fft_extension_stride(hi, stride * 2),
                            || self.das_fft_extension_stride(lo, stride * 2),
                        );
                    } else {
                        self.das_fft_extension_stride(&mut ab[..halfhalf], stride * 2);
                        self.das_fft_extension_stride(&mut ab[halfhalf..], stride * 2);
                    }
                }
                #[cfg(not(feature = "parallel"))]
                {
                    self.das_fft_extension_stride(&mut ab[..halfhalf], stride * 2);
                    self.das_fft_extension_stride(&mut ab[halfhalf..], stride * 2);
                }
                for i in 0..halfhalf {
                    let x = ab[i];
                    let y = ab[halfhalf + i];
                    let y_times_root = y.mul(&self.expanded_roots_of_unity[(1 + 2 * i) * stride]);
                    ab[i] = x.add(&y_times_root);
                    ab[halfhalf + i] = x.sub(&y_times_root);
                }
            }
            Ordering::Equal => {
                let x = ab[0].add(&ab[1]);
                let y = ab[0].sub(&ab[1]);
                let tmp = y.mul(&self.expanded_roots_of_unity[stride]);

                ab[0] = x.add(&tmp);
                ab[1] = x.sub(&tmp);
            }
        }
    }
}

impl DAS<BlstFr> for FFTSettings {
    fn das_fft_extension(&self, vals: &[BlstFr]) -> Result<Vec<BlstFr>, String> {
        if vals.is_empty() {
            return Err(String::from("vals can not be empty"));
        }
        if !vals.len().is_power_of_two() {
            return Err(String::from("vals length has to be power of 2"));
        }
        if vals.len() * 2 > self.max_width {
            return Err(String::from(
                "vals length * 2 has to equal or less than FFTSettings max width",
            ));
        }

        let mut vals = vals.to_vec();
        let stride = self.max_width / (vals.len() * 2);

        self.das_fft_extension_stride(&mut vals, stride);

        let invlen = BlstFr::from_u64(vals.len() as u64);
        let invlen = invlen.inverse();

        for val in &mut vals {
            val.fr *= invlen.fr
        }

        Ok(vals)
    }
}

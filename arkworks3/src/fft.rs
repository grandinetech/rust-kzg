use crate::kzg_proofs::LFFTSettings;
use crate::kzg_types::ArkFr as BlstFr;
use kzg::{FFTFr, Fr as FFr};

impl LFFTSettings {
    /// Fast Fourier Transform for finite field elements, `output` must be zeroes
    pub(crate) fn fft_fr_output(
        &self,
        data: &[BlstFr],
        inverse: bool,
        output: &mut [BlstFr],
    ) -> Result<(), String> {
        if data.len() > self.max_width {
            return Err(String::from(
                "Supplied list is longer than the available max width",
            ));
        }
        if data.len() != output.len() {
            return Err(format!(
                "Output length {} doesn't match data length {}",
                data.len(),
                output.len()
            ));
        }
        if !data.len().is_power_of_two() {
            return Err(String::from("A list with power-of-two length expected"));
        }

        // In case more roots are provided with fft_settings, use a larger stride
        let stride = self.max_width / data.len();

        // Inverse is same as regular, but all constants are reversed and results are divided by n
        // This is a property of the DFT matrix
        let roots = if inverse {
            &self.reverse_roots_of_unity
        } else {
            &self.roots_of_unity
        };

        fft_fr_fast(output, data, 1, roots, stride);

        if inverse {
            let inv_fr_len = BlstFr::from_u64(data.len() as u64).inverse();
            output.iter_mut().for_each(|f| *f = f.mul(&inv_fr_len));
        }

        Ok(())
    }
}

impl FFTFr<BlstFr> for LFFTSettings {
    fn fft_fr(&self, data: &[BlstFr], inverse: bool) -> Result<Vec<BlstFr>, String> {
        let mut ret = vec![BlstFr::default(); data.len()];

        self.fft_fr_output(data, inverse, &mut ret)?;

        Ok(ret)
    }
}

pub fn fft_fr_fast(
    ret: &mut [BlstFr],
    data: &[BlstFr],
    stride: usize,
    roots: &[BlstFr],
    roots_stride: usize,
) {
    let half: usize = ret.len() / 2;
    if half > 0 {
        #[cfg(not(feature = "parallel"))]
        {
            fft_fr_fast(&mut ret[..half], data, stride * 2, roots, roots_stride * 2);
            fft_fr_fast(
                &mut ret[half..],
                &data[stride..],
                stride * 2,
                roots,
                roots_stride * 2,
            );
        }

        #[cfg(feature = "parallel")]
        {
            if half > 256 {
                let (lo, hi) = ret.split_at_mut(half);
                rayon::join(
                    || fft_fr_fast(lo, data, stride * 2, roots, roots_stride * 2),
                    || fft_fr_fast(hi, &data[stride..], stride * 2, roots, roots_stride * 2),
                );
            } else {
                fft_fr_fast(&mut ret[..half], data, stride * 2, roots, roots_stride * 2);
                fft_fr_fast(
                    &mut ret[half..],
                    &data[stride..],
                    stride * 2,
                    roots,
                    roots_stride * 2,
                );
            }
        }

        for i in 0..half {
            let y_times_root = ret[i + half].mul(&roots[i * roots_stride]);
            ret[i + half] = ret[i].sub(&y_times_root);
            ret[i] = ret[i].add(&y_times_root);
        }
    } else {
        ret[0] = data[0];
    }
}

pub fn fft_fr_slow(
    ret: &mut [BlstFr],
    data: &[BlstFr],
    stride: usize,
    roots: &[BlstFr],
    roots_stride: usize,
) {
    let mut v;
    let mut jv;
    let mut r;

    for i in 0..data.len() {
        ret[i] = data[0].mul(&roots[0]);
        for j in 1..data.len() {
            jv = data[j * stride];
            r = roots[((i * j) % data.len()) * roots_stride];
            v = jv.mul(&r);
            ret[i] = ret[i].add(&v);
        }
    }
}

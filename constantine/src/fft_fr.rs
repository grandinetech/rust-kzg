extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use kzg::{FFTFr, Fr};

use crate::types::fft_settings::CtFFTSettings;
use crate::types::fr::CtFr;

/// Fast Fourier Transform for finite field elements. Polynomial ret is operated on in reverse order: ret_i * x ^ (len - i - 1)
pub fn fft_fr_fast(
    ret: &mut [CtFr],
    data: &[CtFr],
    stride: usize,
    roots: &[CtFr],
    roots_stride: usize,
) {
    let half: usize = ret.len() / 2;
    if half > 0 {
        // Recurse
        // Offsetting data by stride = 1 on the first iteration forces the even members to the first half
        // and the odd members to the second half
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
        // When len = 1, return the permuted element
        ret[0] = data[0];
    }
}

impl CtFFTSettings {
    /// Fast Fourier Transform for finite field elements, `output` must be zeroes
    pub(crate) fn fft_fr_output(
        &self,
        data: &[CtFr],
        inverse: bool,
        output: &mut [CtFr],
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
            &self.expanded_roots_of_unity
        };

        fft_fr_fast(output, data, 1, roots, stride);

        if inverse {
            let inv_fr_len = CtFr::from_u64(data.len() as u64).inverse();
            output.iter_mut().for_each(|f| *f = f.mul(&inv_fr_len));
        }

        Ok(())
    }
}

impl FFTFr<CtFr> for CtFFTSettings {
    /// Fast Fourier Transform for finite field elements
    fn fft_fr(&self, data: &[CtFr], inverse: bool) -> Result<Vec<CtFr>, String> {
        let mut ret = vec![CtFr::default(); data.len()];

        self.fft_fr_output(data, inverse, &mut ret)?;

        Ok(ret)
    }
}

/// Simplified Discrete Fourier Transform, mainly used for testing
pub fn fft_fr_slow(
    ret: &mut [CtFr],
    data: &[CtFr],
    stride: usize,
    roots: &[CtFr],
    roots_stride: usize,
) {
    for i in 0..data.len() {
        // Evaluate first member at 1
        ret[i] = data[0].mul(&roots[0]);

        // Evaluate the rest of members using a step of (i * J) % data.len() over the roots
        // This distributes the roots over correct x^n members and saves on multiplication
        for j in 1..data.len() {
            let v = data[j * stride].mul(&roots[((i * j) % data.len()) * roots_stride]);
            ret[i] = ret[i].add(&v);
        }
    }
}

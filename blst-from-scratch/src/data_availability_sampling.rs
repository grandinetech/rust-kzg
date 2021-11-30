use std::cmp::Ordering;
use kzg::{Fr, DAS};

use crate::types::fft_settings::FsFFTSettings;
use crate::types::fr::FsFr;
use crate::utils::is_power_of_two;

// TODO: explain algo
impl FsFFTSettings {
    pub fn das_fft_extension_stride(&self, evens: &mut [FsFr], stride: usize) {
        match evens.len().cmp(&2) {
            Ordering::Less => {
                return;
            }
            Ordering::Equal => {
                let x = evens[0].add(&evens[1]);
                let y = evens[0].sub(&evens[1]);
                let y_times_root = y.mul(&self.expanded_roots_of_unity[stride]);

                evens[0] = x.add(&y_times_root);
                evens[1] = x.sub(&y_times_root);

                return;
            },
            Ordering::Greater => {}
        }

        let half: usize = evens.len() / 2;
        for i in 0..half {
            let tmp1 = evens[i].add(&evens[half + i]);
            let tmp2 = evens[i].sub(&evens[half + i]);
            evens[half + i] = tmp2.mul(&self.reverse_roots_of_unity[i * 2 * stride]);

            evens[i] = tmp1;
        }

        // Recurse
        self.das_fft_extension_stride(&mut evens[..half], stride * 2);
        self.das_fft_extension_stride(&mut evens[half..], stride * 2);

        for i in 0..half {
            let x = evens[i];
            let y = evens[half + i];
            let y_times_root: FsFr = y.mul(&self.expanded_roots_of_unity[(1 + 2 * i) * stride]);

            evens[i] = x.add(&y_times_root);
            evens[half + i] = x.sub(&y_times_root);
        }
    }
}

impl DAS<FsFr> for FsFFTSettings {
    /// Polynomial extension for data availability sampling. Given values of even indices, produce values of odd indices.
    /// FFTSettings must hold at least 2 times the roots of provided evens.
    /// The resulting odd indices make the right half of the coefficients of the inverse FFT of the combined indices zero.
    fn das_fft_extension(&self, evens: &[FsFr]) -> Result<Vec<FsFr>, String> {
        if evens.is_empty() {
            return Err(String::from("A non-zero list ab expected"));
        } else if !is_power_of_two(evens.len()) {
            return Err(String::from("A list with power-of-two length expected"));
        } else if evens.len() * 2 > self.max_width {
            return Err(String::from(
                "Supplied list is longer than the available max width",
            ));
        }

        // In case more roots are provided with fft_settings, use a larger stride
        let stride = self.max_width / (evens.len() * 2);
        let mut odds = evens.to_vec();
        self.das_fft_extension_stride(&mut odds, stride);

        // TODO: explain why each odd member is multiplied by euclidean inverse of length
        let mut inv_len = FsFr::from_u64(odds.len() as u64);
        inv_len = inv_len.eucl_inverse();
        let odds = odds.iter().map(|f| f.mul(&inv_len)).collect();

        Ok(odds)
    }
}

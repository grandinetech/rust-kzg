extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use kzg::{FFTFr, Fr, G1Mul, Poly, FFTG1, G1};

use crate::types::fft_settings::MclFFTSettings;
use crate::types::fr::MclFr;
use crate::types::g1::MclG1;
use crate::types::poly::MclPoly;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

impl MclFFTSettings {
    pub fn toeplitz_part_1(&self, x: &[MclG1]) -> Vec<MclG1> {
        let n = x.len();
        let n2 = n * 2;
        let mut x_ext = Vec::with_capacity(n2);

        x_ext.extend(x.iter().take(n));
        x_ext.resize(n2, MclG1::identity());

        self.fft_g1(&x_ext, false).unwrap()
    }

    /// poly and x_ext_fft should be of same length
    pub fn toeplitz_part_2(&self, poly: &MclPoly, x_ext_fft: &[MclG1]) -> Vec<MclG1> {
        let coeffs_fft = self.fft_fr(&poly.coeffs, false).unwrap();

        #[cfg(feature = "parallel")]
        {
            coeffs_fft
                .into_par_iter()
                .zip(x_ext_fft)
                .take(poly.len())
                .map(|(coeff_fft, x_ext_fft)| x_ext_fft.mul(&coeff_fft))
                .collect()
        }

        #[cfg(not(feature = "parallel"))]
        {
            coeffs_fft
                .into_iter()
                .zip(x_ext_fft)
                .take(poly.len())
                .map(|(coeff_fft, x_ext_fft)| x_ext_fft.mul(&coeff_fft))
                .collect()
        }
    }

    pub fn toeplitz_part_3(&self, h_ext_fft: &[MclG1]) -> Vec<MclG1> {
        let n2 = h_ext_fft.len();
        let n = n2 / 2;

        let mut ret = self.fft_g1(h_ext_fft, true).unwrap();
        ret[n..n2].copy_from_slice(&vec![MclG1::identity(); n2 - n]);

        ret
    }
}

impl MclPoly {
    pub fn toeplitz_coeffs_stride(&self, offset: usize, stride: usize) -> MclPoly {
        let n = self.len();
        let k = n / stride;
        let k2 = k * 2;

        let mut ret = MclPoly::default();
        ret.coeffs.push(self.coeffs[n - 1 - offset]);

        let num_of_zeroes = if k + 2 < k2 { k + 2 - 1 } else { k2 - 1 };
        for _ in 0..num_of_zeroes {
            ret.coeffs.push(MclFr::zero());
        }

        let mut i = k + 2;
        let mut j = 2 * stride - offset - 1;
        while i < k2 {
            ret.coeffs.push(self.coeffs[j]);

            i += 1;
            j += stride;
        }

        ret
    }

    pub fn toeplitz_coeffs_step(&self) -> MclPoly {
        self.toeplitz_coeffs_stride(0, 1)
    }
}

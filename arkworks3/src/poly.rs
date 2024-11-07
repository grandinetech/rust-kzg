use super::kzg_proofs::LFFTSettings;
use super::utils::{PolyData};
use crate::kzg_types::ArkFr as BlstFr;
use crate::utils::pc_poly_into_blst_poly;
use crate::zero_poly::pad_poly;
use ark_bls12_381::Fr;
use ark_poly::univariate::DensePolynomial;
use ark_poly::UVPolynomial;
use ark_std::{log2, Zero};
use kzg::common_utils::{log2_pow2, next_pow_of_2};
use kzg::{FFTFr, FFTSettings as FFTSettingsT, Fr as FrTrait, Poly};
use std::cmp::min;

pub fn poly_flip(input: &PolyData) -> Result<PolyData, String> {
    let mut output = PolyData::new(0);
    for i in 0..input.len() {
        output.coeffs.push(input.coeffs[input.coeffs.len() - i - 1]);
    }
    Ok(output)
}

impl PolyData {
    pub fn _poly_norm(&self) -> Self {
        let mut ret = self.clone();

        let mut temp_len: usize = ret.coeffs.len();
        while temp_len > 0 && ret.coeffs[temp_len - 1].is_zero() {
            temp_len -= 1;
        }

        if temp_len == 0 {
            ret.coeffs = Vec::new();
        } else {
            ret.coeffs = ret.coeffs[0..temp_len].to_vec();
        }

        ret
    }

    pub fn poly_quotient_length(&self, divisor: &Self) -> usize {
        if self.len() >= divisor.len() {
            self.len() - divisor.len() + 1
        } else {
            0
        }
    }

    pub fn pad(&self, out_length: usize) -> Self {
        let mut ret = Self {
            coeffs: vec![BlstFr::zero(); out_length],
        };

        for i in 0..self.len().min(out_length) {
            ret.coeffs[i] = self.coeffs[i];
        }

        ret
    }

    pub fn flip(&self) -> Result<PolyData, String> {
        let mut ret = PolyData {
            coeffs: vec![BlstFr::default(); self.len()],
        };
        for i in 0..self.len() {
            ret.coeffs[i] = self.coeffs[self.coeffs.len() - i - 1]
        }

        Ok(ret)
    }

    pub fn mul_fft(&self, multiplier: &Self, output_len: usize) -> Result<Self, String> {
        let length = next_pow_of_2(self.len() + multiplier.len() - 1);

        let scale = log2_pow2(length);
        let fft_settings = LFFTSettings::new(scale).unwrap();

        let a_pad = self.pad(length);
        let b_pad = multiplier.pad(length);

        let a_fft: Vec<BlstFr>;
        let b_fft: Vec<BlstFr>;

        #[cfg(feature = "parallel")]
        {
            if length > 1024 {
                let mut a_fft_temp = vec![];
                let mut b_fft_temp = vec![];

                rayon::join(
                    || a_fft_temp = fft_settings.fft_fr(&a_pad.coeffs, false).unwrap(),
                    || b_fft_temp = fft_settings.fft_fr(&b_pad.coeffs, false).unwrap(),
                );

                a_fft = a_fft_temp;
                b_fft = b_fft_temp;
            } else {
                a_fft = fft_settings.fft_fr(&a_pad.coeffs, false).unwrap();
                b_fft = fft_settings.fft_fr(&b_pad.coeffs, false).unwrap();
            }
        }

        #[cfg(not(feature = "parallel"))]
        {
            // Convert Poly to values
            a_fft = fft_settings.fft_fr(&a_pad.coeffs, false).unwrap();
            b_fft = fft_settings.fft_fr(&b_pad.coeffs, false).unwrap();
        }

        // Multiply two value ranges
        let mut ab_fft = a_fft;
        ab_fft.iter_mut().zip(b_fft).for_each(|(a, b)| {
            *a = a.mul(&b);
        });

        // Convert value range multiplication to a resulting polynomial
        let ab = fft_settings.fft_fr(&ab_fft, true).unwrap();
        drop(ab_fft);

        let mut ret = PolyData {
            coeffs: vec![BlstFr::zero(); output_len],
        };

        let range = ..output_len.min(length);
        ret.coeffs[range].clone_from_slice(&ab[range]);

        Ok(ret)
    }

    pub fn mul(&mut self, multiplier: &Self, output_len: usize) -> Result<Self, String> {
        if self.len() < 64 || multiplier.len() < 64 || output_len < 128 {
            // Tunable parameter
            self.mul_direct(multiplier, output_len)
        } else {
            self.mul_fft(multiplier, output_len)
        }
    }
}
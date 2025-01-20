extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use kzg::{FFTFr, Fr, PolyRecover, ZeroPoly};

use crate::types::fft_settings::MclFFTSettings;
use crate::types::fr::MclFr;
use crate::types::poly::MclPoly;
use once_cell::sync::OnceCell;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

const SCALE_FACTOR: u64 = 5;
static INVERSE_FACTORS: OnceCell<Vec<MclFr>> = OnceCell::new();
static UNSCALE_FACTOR_POWERS: OnceCell<Vec<MclFr>> = OnceCell::new();

pub fn scale_poly(p: &mut [MclFr], len_p: usize) {
    let factors = INVERSE_FACTORS.get_or_init(|| {
        let scale_factor = MclFr::from_u64(SCALE_FACTOR);
        let inv_factor = MclFr::inverse(&scale_factor);
        let mut temp = Vec::with_capacity(65536);
        temp.push(MclFr::one());
        for i in 1..65536 {
            temp.push(temp[i - 1].mul(&inv_factor));
        }
        temp
    });

    p.iter_mut()
        .zip(factors)
        .take(len_p)
        .skip(1)
        .for_each(|(p, factor)| {
            *p = p.mul(factor);
        });
}

pub fn unscale_poly(p: &mut [MclFr], len_p: usize) {
    let factors = UNSCALE_FACTOR_POWERS.get_or_init(|| {
        let scale_factor = MclFr::from_u64(SCALE_FACTOR);
        let mut temp = Vec::with_capacity(65536);
        temp.push(MclFr::one());
        for i in 1..65536 {
            temp.push(temp[i - 1].mul(&scale_factor));
        }
        temp
    });

    p.iter_mut()
        .zip(factors)
        .take(len_p)
        .skip(1)
        .for_each(|(p, factor)| {
            *p = p.mul(factor);
        });
}

impl PolyRecover<MclFr, MclPoly, MclFFTSettings> for MclPoly {
    fn recover_poly_coeffs_from_samples(
        samples: &[Option<MclFr>],
        fs: &MclFFTSettings,
    ) -> Result<Self, String> {
        let len_samples = samples.len();

        if !len_samples.is_power_of_two() {
            return Err(String::from(
                "Samples must have a length that is a power of two",
            ));
        }

        let mut missing = Vec::with_capacity(len_samples / 2);

        for (i, sample) in samples.iter().enumerate() {
            if sample.is_none() {
                missing.push(i);
            }
        }

        if missing.len() > len_samples / 2 {
            return Err(String::from(
                "Impossible to recover, too many shards are missing",
            ));
        }

        // Calculate `Z_r,I`
        let (zero_eval, mut zero_poly) = fs.zero_poly_via_multiplication(len_samples, &missing)?;

        // Construct E * Z_r,I: the loop makes the evaluation polynomial
        let poly_evaluations_with_zero = samples
            .iter()
            .zip(zero_eval)
            .map(|(maybe_sample, zero_eval)| {
                debug_assert_eq!(maybe_sample.is_none(), zero_eval.is_zero());

                match maybe_sample {
                    Some(sample) => sample.mul(&zero_eval),
                    None => MclFr::zero(),
                }
            })
            .collect::<Vec<_>>();

        // Now inverse FFT so that poly_with_zero is (E * Z_r,I)(x) = (D * Z_r,I)(x)
        let mut poly_with_zero = fs.fft_fr(&poly_evaluations_with_zero, true).unwrap();
        drop(poly_evaluations_with_zero);

        // x -> k * x
        let len_zero_poly = zero_poly.coeffs.len();
        scale_poly(&mut poly_with_zero, len_samples);
        scale_poly(&mut zero_poly.coeffs, len_zero_poly);

        // Q1 = (D * Z_r,I)(k * x)
        let scaled_poly_with_zero = poly_with_zero;

        // Q2 = Z_r,I(k * x)
        let scaled_zero_poly = zero_poly.coeffs;

        // Polynomial division by convolution: Q3 = Q1 / Q2
        #[cfg(feature = "parallel")]
        let (eval_scaled_poly_with_zero, eval_scaled_zero_poly) = {
            if len_zero_poly - 1 > 1024 {
                rayon::join(
                    || fs.fft_fr(&scaled_poly_with_zero, false).unwrap(),
                    || fs.fft_fr(&scaled_zero_poly, false).unwrap(),
                )
            } else {
                (
                    fs.fft_fr(&scaled_poly_with_zero, false).unwrap(),
                    fs.fft_fr(&scaled_zero_poly, false).unwrap(),
                )
            }
        };
        #[cfg(not(feature = "parallel"))]
        let (eval_scaled_poly_with_zero, eval_scaled_zero_poly) = {
            (
                fs.fft_fr(&scaled_poly_with_zero, false).unwrap(),
                fs.fft_fr(&scaled_zero_poly, false).unwrap(),
            )
        };
        drop(scaled_zero_poly);

        let mut eval_scaled_reconstructed_poly = eval_scaled_poly_with_zero;
        #[cfg(not(feature = "parallel"))]
        let eval_scaled_reconstructed_poly_iter = eval_scaled_reconstructed_poly.iter_mut();
        #[cfg(feature = "parallel")]
        let eval_scaled_reconstructed_poly_iter = eval_scaled_reconstructed_poly.par_iter_mut();

        eval_scaled_reconstructed_poly_iter
            .zip(eval_scaled_zero_poly)
            .for_each(
                |(eval_scaled_reconstructed_poly, eval_scaled_poly_with_zero)| {
                    *eval_scaled_reconstructed_poly = eval_scaled_reconstructed_poly
                        .div(&eval_scaled_poly_with_zero)
                        .unwrap();
                },
            );

        // The result of the division is D(k * x):
        let mut scaled_reconstructed_poly =
            fs.fft_fr(&eval_scaled_reconstructed_poly, true).unwrap();
        drop(eval_scaled_reconstructed_poly);

        // k * x -> x
        unscale_poly(&mut scaled_reconstructed_poly, len_samples);

        // Finally we have D(x) which evaluates to our original data at the powers of roots of unity
        Ok(Self {
            coeffs: scaled_reconstructed_poly,
        })
    }

    fn recover_poly_from_samples(
        samples: &[Option<MclFr>],
        fs: &MclFFTSettings,
    ) -> Result<Self, String> {
        let reconstructed_poly = Self::recover_poly_coeffs_from_samples(samples, fs)?;

        // The evaluation polynomial for D(x) is the reconstructed data:
        let reconstructed_data = fs.fft_fr(&reconstructed_poly.coeffs, false).unwrap();

        // Check all is well
        samples
            .iter()
            .zip(&reconstructed_data)
            .for_each(|(sample, reconstructed_data)| {
                debug_assert!(sample.is_none() || reconstructed_data.equals(&sample.unwrap()));
            });

        Ok(Self {
            coeffs: reconstructed_data,
        })
    }
}

use kzg::{FFTFr, Fr, Poly, ZeroPoly};

use crate::types::fft_settings::FsFFTSettings;
use crate::types::fr::FsFr;
use crate::types::poly::FsPoly;
use crate::utils::is_power_of_two;

const SCALE_FACTOR: u64 = 5;
static mut INVERSE_FACTORS: Vec<FsFr> = Vec::new();
static mut UNSCALE_FACTOR_POWERS: Vec<FsFr> = Vec::new();

#[allow(clippy::needless_range_loop)]
pub fn scale_poly(p: &mut Vec<FsFr>, len_p: usize) {
    let scale_factor = FsFr::from_u64(SCALE_FACTOR);
    let inv_factor = FsFr::inverse(&scale_factor);

    unsafe {
        if INVERSE_FACTORS.len() < len_p {
            if INVERSE_FACTORS.is_empty() {
                INVERSE_FACTORS.push(FsFr::one());
            }
            for i in (INVERSE_FACTORS.len())..len_p {
                INVERSE_FACTORS.push(INVERSE_FACTORS[i-1].mul(&inv_factor));
            }
        }

        for i in 1..len_p {
            p[i] = p[i].mul(&INVERSE_FACTORS[i]);
        }
    }
}

#[allow(clippy::needless_range_loop)]
pub fn unscale_poly(p: &mut Vec<FsFr>, len_p: usize) {
    let scale_factor = FsFr::from_u64(SCALE_FACTOR);

    unsafe {
        if UNSCALE_FACTOR_POWERS.len() < len_p {
            if UNSCALE_FACTOR_POWERS.is_empty() {
                UNSCALE_FACTOR_POWERS.push(FsFr::one());
            }
            for i in (UNSCALE_FACTOR_POWERS.len())..len_p {
                UNSCALE_FACTOR_POWERS.push(UNSCALE_FACTOR_POWERS[i-1].mul(&scale_factor));
            }
        }

        for i in 1..len_p {
            p[i] = p[i].mul(&UNSCALE_FACTOR_POWERS[i]);
        }
    }
}

pub fn recover_poly_from_samples(
    samples: &[FsFr],
    len_samples: usize,
    fs: &FsFFTSettings,
) -> Result<Vec<FsFr>, String> {
    if !is_power_of_two(len_samples) {
        return Err(String::from("len_samples must be a power of two"));
    }

    let mut missing: Vec<usize> = Vec::new();
    for (i, sample) in samples.iter().enumerate() {
        if sample.is_null() {
            missing.push(i);
        }
    }

    // Calculate `Z_r,I`
    let (zero_eval, mut zero_poly) = fs
        .zero_poly_via_multiplication(len_samples, &missing)
        .unwrap();

    for i in 0..len_samples {
        if samples[i].is_null() != zero_eval[i].is_zero() {
            return Err(String::from(
                "recovery error: samples should be null when and only when zero_eval is zero",
            ));
        }
    }

    let mut poly_evaluations_with_zero = FsPoly::default();

    // Construct E * Z_r,I: the loop makes the evaluation polynomial
    for i in 0..len_samples {
        if samples[i].is_null() {
            poly_evaluations_with_zero.coeffs.push(FsFr::zero());
        } else {
            poly_evaluations_with_zero
                .coeffs
                .push(samples[i].mul(&zero_eval[i]));
        }
    }
    // Now inverse FFT so that poly_with_zero is (E * Z_r,I)(x) = (D * Z_r,I)(x)
    let mut poly_with_zero: FsPoly = FsPoly::default();
    poly_with_zero.coeffs = fs.fft_fr(&poly_evaluations_with_zero.coeffs, true).unwrap();

    // x -> k * x
    let len_zero_poly = zero_poly.coeffs.len();
    scale_poly(&mut poly_with_zero.coeffs, len_samples);
    scale_poly(&mut zero_poly.coeffs, len_zero_poly);

    // Q1 = (D * Z_r,I)(k * x)
    let scaled_poly_with_zero = poly_with_zero.coeffs;

    // Q2 = Z_r,I(k * x)
    let scaled_zero_poly = zero_poly.coeffs;

    // Polynomial division by convolution: Q3 = Q1 / Q2
    let eval_scaled_poly_with_zero: Vec<FsFr> = fs.fft_fr(&scaled_poly_with_zero, false).unwrap();
    let eval_scaled_zero_poly: Vec<FsFr> = fs.fft_fr(&scaled_zero_poly, false).unwrap();

    let mut eval_scaled_reconstructed_poly = FsPoly::default();
    eval_scaled_reconstructed_poly.coeffs = eval_scaled_poly_with_zero.clone();
    for i in 0..len_samples {
        eval_scaled_reconstructed_poly.coeffs[i] = eval_scaled_poly_with_zero[i]
            .div(&eval_scaled_zero_poly[i])
            .unwrap();
    }

    // The result of the division is D(k * x):
    let mut scaled_reconstructed_poly: Vec<FsFr> = fs
        .fft_fr(&eval_scaled_reconstructed_poly.coeffs, true)
        .unwrap();

    // k * x -> x
    unscale_poly(&mut scaled_reconstructed_poly, len_samples);

    // Finally we have D(x) which evaluates to our original data at the powers of roots of unity
    let reconstructed_poly = scaled_reconstructed_poly;

    // The evaluation polynomial for D(x) is the reconstructed data:
    let reconstructed_data = fs.fft_fr(&reconstructed_poly, false).unwrap();

    // Check all is well
    for i in 0..len_samples {
        if !(samples[i].is_null() || reconstructed_data[i].equals(&samples[i])) {
            return Err(String::from(
                "recovery error: samples should be null or equal reconstructed data",
            ));
        }
    }

    Ok(reconstructed_data)
}

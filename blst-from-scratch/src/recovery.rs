use kzg::{FFTFr, Fr, Poly, ZeroPoly};

use crate::types::fft_settings::FsFFTSettings;
use crate::types::fr::FsFr;
use crate::types::poly::FsPoly;
use crate::utils::{is_power_of_two, next_power_of_two};
use once_cell::sync::OnceCell;

const SCALE_FACTOR: u64 = 5;
static INVERSE_FACTORS: OnceCell<Vec<FsFr>> = OnceCell::new();
static UNSCALE_FACTOR_POWERS: OnceCell<Vec<FsFr>> = OnceCell::new();

#[allow(clippy::needless_range_loop)]
pub fn scale_poly(p: &mut [FsFr], len_p: usize) {
    let factors = INVERSE_FACTORS.get_or_init(|| {
        let scale_factor = FsFr::from_u64(SCALE_FACTOR);
        let inv_factor = FsFr::inverse(&scale_factor);
        let mut temp: Vec<FsFr> = vec![FsFr::one()];
        for i in 1..65536 {
            temp.push(temp[i - 1].mul(&inv_factor));
        }
        temp
    });

    for i in 1..len_p {
        p[i] = p[i].mul(&factors[i]);
    }
}

#[allow(clippy::needless_range_loop)]
pub fn unscale_poly(p: &mut [FsFr], len_p: usize) {
    let factors = UNSCALE_FACTOR_POWERS.get_or_init(|| {
        let scale_factor = FsFr::from_u64(SCALE_FACTOR);
        let mut temp: Vec<FsFr> = vec![FsFr::one()];
        for i in 1..65536 {
            temp.push(temp[i - 1].mul(&scale_factor));
        }
        temp
    });

    for i in 1..len_p {
        p[i] = p[i].mul(&factors[i]);
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

    let _zero_poly_scale = next_power_of_two(len_zero_poly - 1);
    #[cfg(feature = "parallel")]
    {
        if _zero_poly_scale > 1024 {
            rayon::join(
                || scale_poly(&mut poly_with_zero.coeffs, len_samples),
                || scale_poly(&mut zero_poly.coeffs, len_zero_poly),
            );
        } else {
            scale_poly(&mut poly_with_zero.coeffs, len_samples);
            scale_poly(&mut zero_poly.coeffs, len_zero_poly);
        }
    }

    #[cfg(not(feature = "parallel"))]
    {
        scale_poly(&mut poly_with_zero.coeffs, len_samples);
        scale_poly(&mut zero_poly.coeffs, len_zero_poly);
    }

    // Q1 = (D * Z_r,I)(k * x)
    let scaled_poly_with_zero = poly_with_zero.coeffs;

    // Q2 = Z_r,I(k * x)
    let scaled_zero_poly = zero_poly.coeffs;

    #[allow(unused_assignments)]
    let mut eval_scaled_poly_with_zero = vec![];
    #[allow(unused_assignments)]
    let mut eval_scaled_zero_poly = vec![];

    // Polynomial division by convolution: Q3 = Q1 / Q2
    #[cfg(feature = "parallel")]
    {
        if _zero_poly_scale > 1024 {
            rayon::join(
                || eval_scaled_poly_with_zero = fs.fft_fr(&scaled_poly_with_zero, false).unwrap(),
                || eval_scaled_zero_poly = fs.fft_fr(&scaled_zero_poly, false).unwrap(),
            );
        } else {
            eval_scaled_poly_with_zero = fs.fft_fr(&scaled_poly_with_zero, false).unwrap();
            eval_scaled_zero_poly = fs.fft_fr(&scaled_zero_poly, false).unwrap();
        }
    }
    #[cfg(not(feature = "parallel"))]
    {
        eval_scaled_poly_with_zero = fs.fft_fr(&scaled_poly_with_zero, false).unwrap();
        eval_scaled_zero_poly = fs.fft_fr(&scaled_zero_poly, false).unwrap();
    }

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

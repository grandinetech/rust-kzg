use crate::consts::SCALE_FACTOR;
use crate::kzg_proofs::FFTSettings;
use crate::kzg_types::ArkFr as BlstFr;
use crate::utils::PolyData;

use kzg::{FFTFr, Fr, Poly, PolyRecover, ZeroPoly};

#[cfg(feature = "parallel")]
use kzg::common_utils::next_pow_of_2;

#[cfg(feature = "parallel")]
static mut INVERSE_FACTORS: Vec<BlstFr> = Vec::new();
#[cfg(feature = "parallel")]
static mut UNSCALE_FACTOR_POWERS: Vec<BlstFr> = Vec::new();

#[allow(clippy::needless_range_loop)]
pub fn scale_poly(p: &mut PolyData) {
    let scale_factor = BlstFr::from_u64(SCALE_FACTOR);
    let inv_factor = scale_factor.inverse();

    #[allow(static_mut_refs)]
    #[cfg(feature = "parallel")]
    {
        let optim = next_pow_of_2(p.len() - 1);
        if optim <= 1024 {
            unsafe {
                if INVERSE_FACTORS.len() < p.len() {
                    if INVERSE_FACTORS.is_empty() {
                        INVERSE_FACTORS.push(BlstFr::one());
                    }
                    for i in (INVERSE_FACTORS.len())..p.len() {
                        INVERSE_FACTORS.push(INVERSE_FACTORS[i - 1].mul(&inv_factor));
                    }
                }

                for i in 1..p.len() {
                    p.coeffs[i] = p.coeffs[i].mul(&INVERSE_FACTORS[i]);
                }
            }
        } else {
            let mut factor_power = BlstFr::one();
            for i in 1..p.len() {
                factor_power = factor_power.mul(&inv_factor);
                p.set_coeff_at(i, &p.get_coeff_at(i).mul(&factor_power));
            }
        }
    }
    #[cfg(not(feature = "parallel"))]
    {
        let mut factor_power = BlstFr::one();
        for i in 1..p.len() {
            factor_power = factor_power.mul(&inv_factor);
            p.set_coeff_at(i, &p.get_coeff_at(i).mul(&factor_power));
        }
    }
}

#[allow(clippy::needless_range_loop)]
pub fn unscale_poly(p: &mut PolyData) {
    let scale_factor = BlstFr::from_u64(SCALE_FACTOR);

    #[allow(static_mut_refs)]
    #[cfg(feature = "parallel")]
    {
        unsafe {
            if UNSCALE_FACTOR_POWERS.len() < p.len() {
                if UNSCALE_FACTOR_POWERS.is_empty() {
                    UNSCALE_FACTOR_POWERS.push(BlstFr::one());
                }
                for i in (UNSCALE_FACTOR_POWERS.len())..p.len() {
                    UNSCALE_FACTOR_POWERS.push(UNSCALE_FACTOR_POWERS[i - 1].mul(&scale_factor));
                }
            }

            for i in 1..p.len() {
                p.coeffs[i] = p.coeffs[i].mul(&UNSCALE_FACTOR_POWERS[i]);
            }
        }
    }
    #[cfg(not(feature = "parallel"))]
    {
        let mut factor_power = BlstFr::one();
        for i in 1..p.len() {
            factor_power = factor_power.mul(&scale_factor);
            p.set_coeff_at(i, &p.get_coeff_at(i).mul(&factor_power));
        }
    }
}
impl PolyRecover<BlstFr, PolyData, FFTSettings> for PolyData {
    fn recover_poly_coeffs_from_samples(
        samples: &[Option<BlstFr>],
        fs: &FFTSettings,
    ) -> Result<Self, String> {
        if !samples.len().is_power_of_two() {
            return Err(String::from("samples lenght has to be power of 2"));
        }

        let mut missing = Vec::new();

        for (i, sample) in samples.iter().enumerate() {
            if sample.is_none() {
                missing.push(i);
            }
        }

        if missing.len() > samples.len() / 2 {
            return Err(String::from(
                "Impossible to recover, too many shards are missing",
            ));
        }

        // Calculate `Z_r,I`
        let (zero_eval, mut zero_poly) =
            fs.zero_poly_via_multiplication(samples.len(), missing.as_slice())?;

        // Check all is well
        for (i, item) in zero_eval.iter().enumerate().take(samples.len()) {
            if samples[i].is_none() != item.is_zero() {
                return Err(String::from("sample and item are both zero"));
            }
        }

        // Construct E * Z_r,I: the loop makes the evaluation polynomial

        let mut poly_evaluations_with_zero = vec![BlstFr::zero(); samples.len()];

        for i in 0..samples.len() {
            if samples[i].is_none() {
                poly_evaluations_with_zero[i] = BlstFr::zero();
            } else {
                poly_evaluations_with_zero[i] = samples[i].unwrap().mul(&zero_eval[i]);
            }
        }

        // Now inverse FFT so that poly_with_zero is (E * Z_r,I)(x) = (D * Z_r,I)(x)
        let mut poly_with_zero = PolyData {
            coeffs: fs
                .fft_fr(poly_evaluations_with_zero.as_slice(), true)
                .unwrap(),
        };

        #[cfg(feature = "parallel")]
        let optim = next_pow_of_2(poly_with_zero.len() - 1);

        #[cfg(feature = "parallel")]
        {
            if optim > 1024 {
                rayon::join(
                    || scale_poly(&mut poly_with_zero),
                    || scale_poly(&mut zero_poly),
                );
            } else {
                scale_poly(&mut poly_with_zero);
                scale_poly(&mut zero_poly);
            }
        }
        #[cfg(not(feature = "parallel"))]
        {
            scale_poly(&mut poly_with_zero);
            scale_poly(&mut zero_poly);
        }

        // Q1 = (D * Z_r,I)(k * x)
        let scaled_poly_with_zero = poly_with_zero; // Renaming
                                                    // Q2 = Z_r,I(k * x)
        let scaled_zero_poly = zero_poly.coeffs; // Renaming

        let eval_scaled_poly_with_zero;
        let eval_scaled_zero_poly;

        #[cfg(feature = "parallel")]
        {
            if optim > 1024 {
                let mut eval_scaled_poly_with_zero_temp = vec![];
                let mut eval_scaled_zero_poly_temp = vec![];
                rayon::join(
                    || {
                        eval_scaled_poly_with_zero_temp =
                            fs.fft_fr(&scaled_poly_with_zero.coeffs, false).unwrap()
                    },
                    || eval_scaled_zero_poly_temp = fs.fft_fr(&scaled_zero_poly, false).unwrap(),
                );

                eval_scaled_poly_with_zero = eval_scaled_poly_with_zero_temp;
                eval_scaled_zero_poly = eval_scaled_zero_poly_temp;
            } else {
                eval_scaled_poly_with_zero =
                    fs.fft_fr(&scaled_poly_with_zero.coeffs, false).unwrap();
                eval_scaled_zero_poly = fs.fft_fr(&scaled_zero_poly, false).unwrap();
            }
        }
        #[cfg(not(feature = "parallel"))]
        {
            eval_scaled_poly_with_zero = fs.fft_fr(&scaled_poly_with_zero.coeffs, false).unwrap();
            eval_scaled_zero_poly = fs.fft_fr(&scaled_zero_poly, false).unwrap();
        }

        let mut eval_scaled_reconstructed_poly = eval_scaled_poly_with_zero.clone();
        for i in 0..samples.len() {
            eval_scaled_reconstructed_poly[i] = eval_scaled_poly_with_zero[i]
                .div(&eval_scaled_zero_poly[i])
                .unwrap();
        }

        // The result of the division is D(k * x):
        let mut scaled_reconstructed_poly = PolyData {
            coeffs: fs.fft_fr(&eval_scaled_reconstructed_poly, true).unwrap(),
        };

        // k * x -> x
        unscale_poly(&mut scaled_reconstructed_poly);

        // Finally we have D(x) which evaluates to our original data at the powers of roots of unity
        Ok(scaled_reconstructed_poly)
    }

    fn recover_poly_from_samples(
        samples: &[Option<BlstFr>],
        fs: &FFTSettings,
    ) -> Result<Self, String> {
        let reconstructed_poly = Self::recover_poly_coeffs_from_samples(samples, fs)?;

        // The evaluation polynomial for D(x) is the reconstructed data:
        let out = PolyData {
            coeffs: fs.fft_fr(&reconstructed_poly.coeffs, false).unwrap(),
        };

        // Check all is well
        for (i, sample) in samples.iter().enumerate() {
            if !sample.is_none() && !out.get_coeff_at(i).equals(&sample.unwrap()) {
                return Err(String::from(
                    "sample is zero and out coeff at i is not equals to sample",
                ));
            }
        }
        Ok(out)
    }
}

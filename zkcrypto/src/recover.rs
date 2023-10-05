use crate::{fftsettings::ZkFFTSettings, curve::scalar::Scalar};
use crate::poly::ZPoly;
#[cfg(feature = "parallel")]
use crate::zkfr::blsScalar as Scalar;
use kzg::{FFTFr, Fr, Poly, PolyRecover, ZeroPoly};

const SCALE_FACTOR: u64 = 5;

#[cfg(feature = "parallel")]
static mut INVERSE_FACTORS: Vec<Scalar> = Vec::new();
#[cfg(feature = "parallel")]
static mut UNSCALE_FACTOR_POWERS: Vec<Scalar> = Vec::new();

#[allow(clippy::needless_range_loop)]
pub fn scale_poly(p: &mut ZPoly) {
    let scale_factor = Scalar::from_u64(SCALE_FACTOR);
    let inv_factor = scale_factor.inverse();

    #[cfg(feature = "parallel")]
    {
        let optim = next_pow_of_2(p.len() - 1);
        if optim <= 1024 {
            unsafe {
                if INVERSE_FACTORS.len() < p.len() {
                    if INVERSE_FACTORS.is_empty() {
                        INVERSE_FACTORS.push(Scalar::one());
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
            let mut factor_power = Scalar::one();
            for i in 1..p.len() {
                factor_power = factor_power.mul(&inv_factor);
                p.set_coeff_at(i, &p.get_coeff_at(i).mul(&factor_power));
            }
        }
    }

    #[cfg(not(feature = "parallel"))]
    {
        let mut factor_power = Scalar::one();
        for i in 1..p.len() {
            factor_power = factor_power.mul(&inv_factor);
            p.set_coeff_at(i, &p.get_coeff_at(i).mul(&factor_power));
        }
    }
}

#[allow(clippy::needless_range_loop)]
pub fn unscale_poly(p: &mut ZPoly) {
    let scale_factor = Scalar::from_u64(SCALE_FACTOR);

    #[cfg(feature = "parallel")]
    {
        unsafe {
            if UNSCALE_FACTOR_POWERS.len() < p.len() {
                if UNSCALE_FACTOR_POWERS.is_empty() {
                    UNSCALE_FACTOR_POWERS.push(Scalar::one());
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
        let mut factor_power = Scalar::one();
        for i in 1..p.len() {
            factor_power = factor_power.mul(&scale_factor);
            p.set_coeff_at(i, &p.get_coeff_at(i).mul(&factor_power));
        }
    }
}

impl PolyRecover<Scalar, ZPoly, ZkFFTSettings> for ZPoly {
    fn recover_poly_coeffs_from_samples(
        samples: &[Option<Scalar>],
        fs: &ZkFFTSettings,
    ) -> Result<Self, String> {
        assert!(samples.len().is_power_of_two());
        let mut missing = Vec::new();

        for (i, sample) in samples.iter().enumerate() {
            if sample.is_none() {
                // len_missing+= 1;
                missing.push(i);
            }
        }

        if missing.len() > samples.len() / 2 {
            return Err(String::from(
                "Impossible to recover, too many shards are missing",
            ));
        }

        let (zero_eval, mut zero_poly) =
            fs.zero_poly_via_multiplication(samples.len(), missing.as_slice())?;

        for (i, item) in zero_eval.iter().enumerate().take(samples.len()) {
            assert!(samples[i].is_none() == item.is_zero());
        }

        let mut poly_evaluations_with_zero = vec![Scalar::zero(); samples.len()];

        for i in 0..samples.len() {
            if samples[i].is_none() {
                poly_evaluations_with_zero[i] = Scalar::zero();
            } else {
                poly_evaluations_with_zero[i] = samples[i].unwrap().mul(&zero_eval[i]);
            }
        }

        let mut poly_with_zero = ZPoly {
            coeffs: fs
                .fft_fr(poly_evaluations_with_zero.as_slice(), true)
                .unwrap(),
        };

        #[cfg(feature = "parallel")]
        {
            let optim = next_pow_of_2(poly_with_zero.len() - 1);

            if optim > 1024 {
                rayon::join(
                    || scale_poly(&mut poly_with_zero),
                    || scale_poly(&mut zero_poly),
                );
            } else {
                scale_poly(&mut poly_with_zero);
                scale_poly(&mut zero_poly);
            }

            let scaled_poly_with_zero = poly_with_zero;
            let scaled_zero_poly = zero_poly.coeffs;

            let mut eval_scaled_poly_with_zero = vec![];
            let mut eval_scaled_zero_poly = vec![];

            if optim > 1024 {
                rayon::join(
                    || {
                        eval_scaled_poly_with_zero =
                            fs.fft_fr(&scaled_poly_with_zero.coeffs, false).unwrap()
                    },
                    || eval_scaled_zero_poly = fs.fft_fr(&scaled_zero_poly, false).unwrap(),
                );
            } else {
                eval_scaled_poly_with_zero =
                    fs.fft_fr(&scaled_poly_with_zero.coeffs, false).unwrap();
                eval_scaled_zero_poly = fs.fft_fr(&scaled_zero_poly, false).unwrap();
            }

            let mut eval_scaled_reconstructed_poly = eval_scaled_poly_with_zero.clone();
            for i in 0..samples.len() {
                eval_scaled_reconstructed_poly[i] = eval_scaled_poly_with_zero[i]
                    .div(&eval_scaled_zero_poly[i])
                    .unwrap();
            }

            let mut scaled_reconstructed_poly = ZPoly {
                coeffs: fs.fft_fr(&eval_scaled_reconstructed_poly, true).unwrap(),
            };
            unscale_poly(&mut scaled_reconstructed_poly);

            Ok(scaled_reconstructed_poly)
        }

        #[cfg(not(feature = "parallel"))]
        {
            scale_poly(&mut poly_with_zero);
            scale_poly(&mut zero_poly);

            let scaled_poly_with_zero = poly_with_zero;
            let scaled_zero_poly = zero_poly.coeffs;

            let eval_scaled_poly_with_zero =
                fs.fft_fr(&scaled_poly_with_zero.coeffs, false).unwrap();
            let eval_scaled_zero_poly = fs.fft_fr(&scaled_zero_poly, false).unwrap();

            let mut eval_scaled_reconstructed_poly = eval_scaled_poly_with_zero.clone();
            for i in 0..samples.len() {
                eval_scaled_reconstructed_poly[i] = eval_scaled_poly_with_zero[i]
                    .div(&eval_scaled_zero_poly[i])
                    .unwrap();
            }

            let mut scaled_reconstructed_poly = ZPoly {
                coeffs: fs.fft_fr(&eval_scaled_reconstructed_poly, true).unwrap(),
            };
            unscale_poly(&mut scaled_reconstructed_poly);

            Ok(scaled_reconstructed_poly)
        }
    }

    fn recover_poly_from_samples(
        samples: &[Option<Scalar>],
        fs: &ZkFFTSettings,
    ) -> Result<Self, String> {
        let reconstructed_poly = Self::recover_poly_coeffs_from_samples(samples, fs)?;
        let out = ZPoly {
            coeffs: fs.fft_fr(&reconstructed_poly.coeffs, false).unwrap(),
        };

        for (i, sample) in samples.iter().enumerate() {
            assert!(sample.is_none() || out.get_coeff_at(i).equals(&sample.unwrap()));
        }
        Ok(out)
    }
}

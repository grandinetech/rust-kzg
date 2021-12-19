use crate::utils::next_power_of_two;
use crate::zkfr::blsScalar as Scalar;
use crate::poly::ZPoly;
use crate::fftsettings::{ZkFFTSettings};
use kzg::{Fr, Poly, ZeroPoly, FFTFr, PolyRecover};

const SCALE_FACTOR: u64 = 5;

pub fn scale_poly(p: &mut ZPoly){
    let scale_factor = Scalar::from_u64(SCALE_FACTOR);
    let inv_factor = scale_factor.inverse();
    let mut factor_power = Scalar::one();

    for i in 1..p.len(){
        factor_power = factor_power.mul(&inv_factor);
        p.set_coeff_at(i, &p.get_coeff_at(i).mul(&factor_power));
    }
}

pub fn unscale_poly(p: &mut ZPoly) {
    let scale_factor = Scalar::from_u64(SCALE_FACTOR);
    let mut factor_power = Scalar::one();

    for i in 1..p.len(){
        factor_power = factor_power.mul(&scale_factor);
        p.set_coeff_at(i, &p.get_coeff_at(i).mul(&factor_power));
    }
}

impl PolyRecover<Scalar, ZPoly, ZkFFTSettings> for ZPoly{
    fn recover_poly_from_samples(samples: &[Option<Scalar>], fs: &ZkFFTSettings) -> Result<Self, String> {
        assert!(samples.len().is_power_of_two());
        let mut missing = Vec::new();

        for (i, sample) in samples.iter().enumerate() {
            if sample.is_none() {
                // len_missing+= 1;
                missing.push(i);
            }
        }

        let (zero_eval, mut zero_poly) = fs.zero_poly_via_multiplication(samples.len(), missing.as_slice()).unwrap();

        for (i, item) in zero_eval.iter().enumerate().take(samples.len()) {
            assert!(samples[i].is_none() == item.is_zero());
        }

        let mut poly_evaluations_with_zero = vec![Scalar::zero();samples.len()];

        for i in 0..samples.len() {
            if samples[i].is_none(){
                poly_evaluations_with_zero[i] = Scalar::zero();
            } else {
                poly_evaluations_with_zero[i] = samples[i].unwrap().mul(&zero_eval[i]);
            }
        }

        let mut poly_with_zero = ZPoly{coeffs:fs.fft_fr(poly_evaluations_with_zero.as_slice(), true).unwrap()};

        #[cfg(feature = "parallel")] {
            let optim = next_power_of_two(poly_with_zero.len() - 1);

            if optim > 1024 {
                rayon::join(
                    || scale_poly(&mut poly_with_zero),
                    || scale_poly(&mut zero_poly),
                );
            }
            else {
                scale_poly(&mut poly_with_zero);
                scale_poly(&mut zero_poly);
            }
        }

        #[cfg(not(feature = "parallel"))] {
            scale_poly(&mut poly_with_zero);
            scale_poly(&mut zero_poly);
        }

        let scaled_poly_with_zero = poly_with_zero;
        let scaled_zero_poly = zero_poly.coeffs; // Renaming

        #[cfg(feature = "parallel")] {
            let mut eval_scaled_poly_with_zero = vec![];
            let mut eval_scaled_zero_poly = vec![];

            if optim > 1024 {
                rayon::join(
                    || eval_scaled_poly_with_zero = fs.fft_fr(&scaled_poly_with_zero.coeffs, false).unwrap(),
                    || eval_scaled_zero_poly = fs.fft_fr(&scaled_zero_poly, false).unwrap(),
                );
            }
            else {
                eval_scaled_poly_with_zero = fs.fft_fr(&scaled_poly_with_zero.coeffs, false).unwrap();
                eval_scaled_zero_poly = fs.fft_fr(&scaled_zero_poly, false).unwrap();
            }
        }

        #[cfg(not(feature = "parallel"))] {
            let eval_scaled_poly_with_zero = fs.fft_fr(&scaled_poly_with_zero.coeffs, false).unwrap();
            let eval_scaled_zero_poly = fs.fft_fr(&scaled_zero_poly, false).unwrap();
        }
        
        let mut eval_scaled_reconstructed_poly = eval_scaled_poly_with_zero.clone();
        for i in 0..samples.len() {
            eval_scaled_reconstructed_poly[i] = eval_scaled_poly_with_zero[i].div(&eval_scaled_zero_poly[i]).unwrap();
        }

        let mut scaled_reconstructed_poly = ZPoly{coeffs:fs.fft_fr(&eval_scaled_reconstructed_poly, true).unwrap()};
        unscale_poly(&mut scaled_reconstructed_poly);

        let reconstructed_poly = scaled_reconstructed_poly; // Renaming
        let out = ZPoly{coeffs: fs.fft_fr(&reconstructed_poly.coeffs, false).unwrap()};

        for (i, sample) in samples.iter().enumerate() {
            assert!(sample.is_none() || out.get_coeff_at(i).equals(&sample.unwrap()));
        }
        Ok(out)
    }
}

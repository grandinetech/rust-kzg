use crate::utils::PolyData;
use crate::kzg_types::{FsFr as BlstFr};
use crate::kzg_proofs::FFTSettings;
use kzg::{Fr, Poly, ZeroPoly, FFTFr, PolyRecover};

const SCALE_FACTOR: u64 = 5;

pub fn scale_poly(p: &mut PolyData){
    let scale_factor = BlstFr::from_u64(SCALE_FACTOR);
    let inv_factor = scale_factor.inverse();
    let mut factor_power = BlstFr::one();

    for i in 1..p.len(){
        factor_power = factor_power.mul(&inv_factor);
        p.set_coeff_at(i, &p.get_coeff_at(i).mul(&factor_power));
    }
}

pub fn unscale_poly(p: &mut PolyData) {
    let scale_factor = BlstFr::from_u64(SCALE_FACTOR);
    let mut factor_power = BlstFr::one();

    for i in 1..p.len(){
        factor_power = factor_power.mul(&scale_factor);
        p.set_coeff_at(i, &p.get_coeff_at(i).mul(&factor_power));
    }
}
impl PolyRecover<BlstFr, PolyData, FFTSettings> for PolyData{
    fn recover_poly_from_samples(samples: &[Option<BlstFr>], fs: &FFTSettings) -> Result<Self, String> {

        assert!(samples.len().is_power_of_two());

        let mut missing = Vec::new();

        // let mut len_missing = 0;
        for (i, sample) in samples.iter().enumerate() {
            if sample.is_none() {
                // len_missing+= 1;
                missing.push(i);
            }
        }

        // Calculate `Z_r,I`
        // TRY(zero_polynomial_via_multiplication(zero_eval, &zero_poly, len_samples, missing, len_missing, fs));
        let (zero_eval, mut zero_poly) = fs.zero_poly_via_multiplication(samples.len(), missing.as_slice()).unwrap();

        // Check all is well
        for (i, item) in zero_eval.iter().enumerate().take(samples.len()) {
            assert!(samples[i].is_none() == item.is_zero());
        }

        // Construct E * Z_r,I: the loop makes the evaluation polynomial

        let mut poly_evaluations_with_zero = vec![BlstFr::zero();samples.len()];

        for i in 0..samples.len() {
            if samples[i].is_none(){
                poly_evaluations_with_zero[i] = BlstFr::zero();
            } else {
                poly_evaluations_with_zero[i] = samples[i].unwrap().mul(&zero_eval[i]);
            }
        }
        // Now inverse FFT so that poly_with_zero is (E * Z_r,I)(x) = (D * Z_r,I)(x)
        // TRY(fft_fr(poly_with_zero, poly_evaluations_with_zero, true, len_samples, fs));
        let mut poly_with_zero = PolyData{coeffs:fs.fft_fr(poly_evaluations_with_zero.as_slice(), true).unwrap()};

        // x -> k * x
        scale_poly(&mut poly_with_zero);
        scale_poly(&mut zero_poly);

        // Q1 = (D * Z_r,I)(k * x)
        let scaled_poly_with_zero = poly_with_zero; // Renaming
        // Q2 = Z_r,I(k * x)
        let scaled_zero_poly = zero_poly.coeffs; // Renaming

        // Polynomial division by convolution: Q3 = Q1 / Q2
        // TRY(fft_fr(eval_scaled_poly_with_zero, scaled_poly_with_zero, false, len_samples, fs));
        let eval_scaled_poly_with_zero = fs.fft_fr(&scaled_poly_with_zero.coeffs, false).unwrap();
        // TRY(fft_fr(eval_scaled_zero_poly, scaled_zero_poly, false, len_samples, fs));
        let eval_scaled_zero_poly = fs.fft_fr(&scaled_zero_poly, false).unwrap();

        let mut eval_scaled_reconstructed_poly = eval_scaled_poly_with_zero.clone();
        for i in 0..samples.len() {
            eval_scaled_reconstructed_poly[i] = eval_scaled_poly_with_zero[i].div(&eval_scaled_zero_poly[i]).unwrap();
        }

        // The result of the division is D(k * x):
        // TRY(fft_fr(scaled_reconstructed_poly, eval_scaled_reconstructed_poly, true, len_samples, fs));
        let mut scaled_reconstructed_poly = PolyData{coeffs:fs.fft_fr(&eval_scaled_reconstructed_poly, true).unwrap()};

        // k * x -> x
        unscale_poly(&mut scaled_reconstructed_poly);

        // Finally we have D(x) which evaluates to our original data at the powers of roots of unity
        let reconstructed_poly = scaled_reconstructed_poly; // Renaming

        // The evaluation polynomial for D(x) is the reconstructed data:
        // TRY(fft_fr(reconstructed_data, reconstructed_poly, false, len_samples, fs));
        let out = PolyData{coeffs: fs.fft_fr(&reconstructed_poly.coeffs, false).unwrap()};
        // reconstructed_data.coeffs = fs.fft_fr(&reconstructed_poly.coeffs, false).unwrap();

        // Check all is well
        for (i, sample) in samples.iter().enumerate() {
            assert!(sample.is_none() || out.get_coeff_at(i).equals(&sample.unwrap()));
        }
        Ok(out)
    }
}

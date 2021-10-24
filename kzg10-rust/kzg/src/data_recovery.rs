use crate::kzg10::*;
use crate::fk20_fft::*;
use crate::data_types::fr::Fr;

// Data recovery

impl Polynomial {
    pub fn shift_in_place(&mut self) {
        self._shift_in_place(&Fr::from_int(PRIMITIVE_ROOT));
    }

    pub fn unshift_in_place(&mut self) {
        self._shift_in_place(&Fr::from_int(PRIMITIVE_ROOT).get_inv());
    }

    //TODO, use precalculated tables for factors?
    fn _shift_in_place(&mut self, factor: &Fr){
        let mut factor_to_power = Fr::one();
        for i in 0..self.order() {
            self.coeffs[i] *= &factor_to_power;
            factor_to_power *= factor;
        }
    }

    pub fn recover_from_samples(fft_settings: FFTSettings, samples: &[Option<Fr>]) -> Polynomial {
        let missing_data_indices: Vec<usize> = samples.iter()
            .enumerate()
            .filter(|(_, ex)| ex.is_none())
            .map(|(ix, _)| ix)
            .collect();

        let (zero_eval, zero_poly_coeffs) = fft_settings.zero_poly_via_multiplication(&missing_data_indices, samples.len());

        // TODO: possible optimization, remove clone()
        let poly_evals_with_zero: Vec<Fr> = samples.iter()
            .zip(zero_eval)
            .map(|(x, eval)| {
                if x.is_none() {
                    return Fr::zero();
                }
                return &x.clone().unwrap() * &eval;
            }).collect();

        let poly_with_zero_coeffs = fft_settings.fft(&poly_evals_with_zero, true);
        let mut poly_with_zero = Polynomial::from_fr(poly_with_zero_coeffs);
        poly_with_zero.shift_in_place();

        let mut zero_poly = Polynomial::from_fr(zero_poly_coeffs);
        zero_poly.shift_in_place();

        let eval_shifted_poly_with_zero = fft_settings.fft(&poly_with_zero.coeffs, false);
        let eval_shifted_zero_poly = fft_settings.fft(&zero_poly.coeffs, false);
        
    
        let eval_shifted_reconstructed_poly: Vec<Fr> = eval_shifted_poly_with_zero.iter()
            .zip(eval_shifted_zero_poly)
            .map(|(a, b)| a / &b)
            .collect();

        let shifted_reconstructed_poly_coeffs = fft_settings.fft(&eval_shifted_reconstructed_poly, true);
        let mut shifted_reconstructed_poly = Polynomial::from_fr(shifted_reconstructed_poly_coeffs);
        shifted_reconstructed_poly.unshift_in_place();

        let reconstructed_data = fft_settings.fft(&shifted_reconstructed_poly.coeffs, false);
        
        return Polynomial::from_fr(reconstructed_data);
    }

    pub fn unwrap_default(values: &Vec<Option<Fr>>) -> Vec<Fr> {
        return values.iter().map(|x| {
            if x.is_none() {
                return Fr::zero()
            }
            return x.clone().unwrap();
        }).collect();
    }
}
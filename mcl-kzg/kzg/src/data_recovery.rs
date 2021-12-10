use crate::kzg10::*;
use crate::fk20_fft::*;
use crate::data_types::fr::Fr;
use crate::utilities::is_power_of_2;

#[cfg(feature = "parallel")]
static mut INVERSE_FACTORS: Vec<Fr> = Vec::new();
#[cfg(feature = "parallel")]
static mut UNSHIFT_FACTOR_POWERS: Vec<Fr> = Vec::new();

impl Polynomial {
    // #[cfg(feature = "parallel")] 
    pub fn shift_in_place(&mut self) {
        let inv_factor = Fr::from_int(PRIMITIVE_ROOT).get_inv();
        #[cfg(feature = "parallel")]
        {
            unsafe {
                if INVERSE_FACTORS.len() < self.order() {
                    if INVERSE_FACTORS.is_empty() {
                        INVERSE_FACTORS.push(Fr::one());
                    }
                    for i in (INVERSE_FACTORS.len())..self.order() {
                        let mut res = Fr::zero();
                        Fr::mul(&mut res, &INVERSE_FACTORS[i-1], &inv_factor);
                        INVERSE_FACTORS.push(res);
                    }
                }
    
                for i in 1..self.order() {
                    self.coeffs[i] *= &INVERSE_FACTORS[i];
                }
            }
        }
        #[cfg(not(feature="parallel"))]
        {
            self._shift_in_place(&inv_factor);
        }
    }

    // #[cfg(not(feature="parallel"))]
    // pub fn shift_in_place(&mut self) {
    //     self._shift_in_place(&Fr::from_int(PRIMITIVE_ROOT).get_inv());
    // }

    
    // #[cfg(feature = "parallel")] 
    pub fn unshift_in_place(&mut self) {
        let scale_factor = Fr::from_int(PRIMITIVE_ROOT);
        #[cfg(feature = "parallel")]
        {
            unsafe {
                if UNSHIFT_FACTOR_POWERS.len() < self.order() {
                    if UNSHIFT_FACTOR_POWERS.is_empty() {
                        UNSHIFT_FACTOR_POWERS.push(Fr::one());
                    }
                    for i in (UNSHIFT_FACTOR_POWERS.len())..self.order() {
                        let mut res = Fr::zero();
                        Fr::mul(&mut res, &UNSHIFT_FACTOR_POWERS[i-1], &scale_factor);
                        UNSHIFT_FACTOR_POWERS.push(res);
                    }
                }
    
                for i in 1..self.order() {
                    self.coeffs[i] *= &UNSHIFT_FACTOR_POWERS[i];
                }
            }
        }
        #[cfg(not(feature="parallel"))]
        {
            self._shift_in_place(&scale_factor);
        }
    }

    // #[cfg(not(feature="parallel"))]
    // pub fn unshift_in_place(&mut self) {
    //     self._shift_in_place(&Fr::from_int(PRIMITIVE_ROOT));
    // }

    //TODO, use precalculated tables for factors?
    fn _shift_in_place(&mut self, factor: &Fr){
        let mut factor_to_power = Fr::one();
        for i in 0..self.order() {
            self.coeffs[i] *= &factor_to_power;
            factor_to_power *= factor;
        }
    }

    pub fn recover_from_samples(fft_settings: &FFTSettings, samples: &[Option<Fr>]) -> Result<Self, String> {
        if !is_power_of_2(samples.len()) {
            return Err(String::from("length of samples must be a power of two"));
        }

        let missing_data_indices: Vec<usize> = samples.iter()
            .enumerate()
            .filter(|(_, ex)| ex.is_none())
            .map(|(ix, _)| ix)
            .collect();

        let (zero_eval, zero_poly_coeffs) = fft_settings.zero_poly_via_multiplication(samples.len(), &missing_data_indices).unwrap();

        // TODO: possible optimization, remove clone()
        let poly_evals_with_zero: Vec<Fr> = samples.iter()
            .zip(zero_eval)
            .map(|(x, eval)| {
                if x.is_none() {
                    return Fr::zero();
                }
                (*x).unwrap() * eval
            }).collect();

        let poly_with_zero_coeffs = fft_settings.fft(&poly_evals_with_zero, true).unwrap();
        let mut poly_with_zero = Polynomial::from_fr(poly_with_zero_coeffs);
        poly_with_zero.shift_in_place();

        let mut zero_poly = Polynomial::from_fr(zero_poly_coeffs.coeffs);
        zero_poly.shift_in_place();

        let eval_shifted_poly_with_zero = fft_settings.fft(&poly_with_zero.coeffs, false).unwrap();
        let eval_shifted_zero_poly = fft_settings.fft(&zero_poly.coeffs, false).unwrap();
        
    
        let eval_shifted_reconstructed_poly: Vec<Fr> = eval_shifted_poly_with_zero.iter()
            .zip(eval_shifted_zero_poly)
            .map(|(a, b)| a / &b)
            .collect();

        let shifted_reconstructed_poly_coeffs = fft_settings.fft(&eval_shifted_reconstructed_poly, true).unwrap();
        let mut shifted_reconstructed_poly = Polynomial::from_fr(shifted_reconstructed_poly_coeffs);
        shifted_reconstructed_poly.unshift_in_place();

        let reconstructed_data = fft_settings.fft(&shifted_reconstructed_poly.coeffs, false).unwrap();
        
        Ok(Polynomial::from_fr(reconstructed_data))
    }

    pub fn unwrap_default(values: &[Option<Fr>]) -> Vec<Fr> {
        return values.iter().map(|x| {
            if x.is_none() {
                return Fr::zero()
            }
            (*x).unwrap()
        }).collect();
    }
}
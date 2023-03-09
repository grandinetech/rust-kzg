use crate::data_types::fr::Fr;
use crate::fk20_fft::*;
use crate::kzg10::*;
use crate::utilities::is_power_of_2;
#[cfg(feature = "parallel")]
use crate::utilities::next_pow_of_2;
#[cfg(feature = "parallel")]
use once_cell::sync::OnceCell;

#[cfg(feature = "parallel")]
static INVERSE_FACTORS: OnceCell<Vec<Fr>> = OnceCell::new();
#[cfg(feature = "parallel")]
static UNSHIFT_FACTOR_POWERS: OnceCell<Vec<Fr>> = OnceCell::new();

impl Polynomial {
    #[allow(clippy::needless_range_loop)]
    pub fn shift_in_place(&mut self) {
        let inv_factor = Fr::from_int(PRIMITIVE_ROOT).get_inv();
        #[cfg(feature = "parallel")]
        {
            let factors = INVERSE_FACTORS.get_or_init(|| {
                let mut temp: Vec<Fr> = vec![Fr::one()];
                for i in 1..65536 {
                    let mut res = Fr::zero();
                    Fr::mul(&mut res, &temp[i - 1], &inv_factor);

                    temp.push(res);
                }

                temp
            });

            for i in 1..self.order() {
                self.coeffs[i] *= &factors[i];
            }
        }
        #[cfg(not(feature = "parallel"))]
        {
            self._shift_in_place(&inv_factor);
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn unshift_in_place(&mut self) {
        let scale_factor = Fr::from_int(PRIMITIVE_ROOT);
        #[cfg(feature = "parallel")]
        {
            let factors = UNSHIFT_FACTOR_POWERS.get_or_init(|| {
                let mut temp: Vec<Fr> = vec![Fr::one()];
                for i in 1..65536 {
                    let mut res = Fr::zero();
                    Fr::mul(&mut res, &temp[i - 1], &scale_factor);

                    temp.push(res);
                }

                temp
            });

            for i in 1..self.order() {
                self.coeffs[i] *= &factors[i];
            }
        }
        #[cfg(not(feature = "parallel"))]
        {
            self._shift_in_place(&scale_factor);
        }
    }

    //TODO, use precalculated tables for factors?
    fn _shift_in_place(&mut self, factor: &Fr) {
        let mut factor_to_power = Fr::one();
        for i in 0..self.order() {
            self.coeffs[i] *= &factor_to_power;
            factor_to_power *= factor;
        }
    }

    pub fn recover_from_samples(
        fft_settings: &FFTSettings,
        samples: &[Option<Fr>],
    ) -> Result<Self, String> {
        if !is_power_of_2(samples.len()) {
            return Err(String::from("length of samples must be a power of two"));
        }

        let missing_data_indices: Vec<usize> = samples
            .iter()
            .enumerate()
            .filter(|(_, ex)| ex.is_none())
            .map(|(ix, _)| ix)
            .collect();

        let (zero_eval, zero_poly_coeffs) = fft_settings
            .zero_poly_via_multiplication(samples.len(), &missing_data_indices)
            .unwrap();

        // TODO: possible optimization, remove clone()
        let poly_evals_with_zero: Vec<Fr> = samples
            .iter()
            .zip(zero_eval)
            .map(|(x, eval)| {
                if x.is_none() {
                    return Fr::zero();
                }
                (*x).unwrap() * eval
            })
            .collect();

        let poly_with_zero_coeffs = fft_settings.fft(&poly_evals_with_zero, true).unwrap();
        let mut poly_with_zero = Polynomial::from_fr(poly_with_zero_coeffs);
        let mut zero_poly = Polynomial::from_fr(zero_poly_coeffs.coeffs);

        #[cfg(feature = "parallel")]
        let optim = next_pow_of_2(poly_with_zero.order() - 1);

        #[cfg(feature = "parallel")]
        {
            if optim > 1024 {
                rayon::join(
                    || poly_with_zero.shift_in_place(),
                    || zero_poly.shift_in_place(),
                );
            } else {
                poly_with_zero.shift_in_place();
                zero_poly.shift_in_place();
            }
        }

        #[cfg(not(feature = "parallel"))]
        {
            poly_with_zero.shift_in_place();
            zero_poly.shift_in_place();
        }

        let eval_shifted_poly_with_zero: Vec<Fr>;
        let eval_shifted_zero_poly: Vec<Fr>;

        #[cfg(not(feature = "parallel"))]
        {
            eval_shifted_poly_with_zero = fft_settings.fft(&poly_with_zero.coeffs, false).unwrap();
            eval_shifted_zero_poly = fft_settings.fft(&zero_poly.coeffs, false).unwrap();
        }

        #[cfg(feature = "parallel")]
        {
            if optim > 1024 {
                let mut eval_shifted_poly_with_zero_temp = vec![];
                let mut eval_shifted_zero_poly_temp = vec![];

                rayon::join(
                    || {
                        eval_shifted_poly_with_zero_temp =
                            fft_settings.fft(&poly_with_zero.coeffs, false).unwrap()
                    },
                    || {
                        eval_shifted_zero_poly_temp =
                            fft_settings.fft(&zero_poly.coeffs, false).unwrap()
                    },
                );

                eval_shifted_poly_with_zero = eval_shifted_poly_with_zero_temp;
                eval_shifted_zero_poly = eval_shifted_zero_poly_temp;
            } else {
                eval_shifted_poly_with_zero =
                    fft_settings.fft(&poly_with_zero.coeffs, false).unwrap();
                eval_shifted_zero_poly = fft_settings.fft(&zero_poly.coeffs, false).unwrap();
            }
        }

        let eval_shifted_reconstructed_poly: Vec<Fr> = eval_shifted_poly_with_zero
            .iter()
            .zip(eval_shifted_zero_poly)
            .map(|(a, b)| a / &b)
            .collect();

        let shifted_reconstructed_poly_coeffs = fft_settings
            .fft(&eval_shifted_reconstructed_poly, true)
            .unwrap();
        let mut shifted_reconstructed_poly = Polynomial::from_fr(shifted_reconstructed_poly_coeffs);
        shifted_reconstructed_poly.unshift_in_place();

        let reconstructed_data = fft_settings
            .fft(&shifted_reconstructed_poly.coeffs, false)
            .unwrap();

        Ok(Polynomial::from_fr(reconstructed_data))
    }

    pub fn unwrap_default(values: &[Option<Fr>]) -> Vec<Fr> {
        return values
            .iter()
            .map(|x| {
                if x.is_none() {
                    return Fr::zero();
                }
                (*x).unwrap()
            })
            .collect();
    }
}

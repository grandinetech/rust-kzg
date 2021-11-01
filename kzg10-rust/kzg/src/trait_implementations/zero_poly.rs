use crate::data_types::fr::Fr;
use crate::fk20_fft::FFTSettings;
use crate::kzg10::Polynomial;
use kzg::ZeroPoly;

impl ZeroPoly<Fr, Polynomial> for FFTSettings {
    fn do_zero_poly_mul_partial(&self, _idxs: &[usize], _stride: usize) -> Result<Polynomial, String> {
        todo!("I'm not sure this is applicable for our team")
    }

    fn reduce_partials(&self, _domain_size: usize, _partials: &[Polynomial]) -> Result<Polynomial, String> {
       todo!("I'm not sure this is applicable for our team")
    }
    
    fn zero_poly_via_multiplication(&self, domain_size: usize, missing_idxs: &[usize]) -> Result<(Vec<Fr>, Polynomial), String> {
        let (zero_eval, zero_poly) = FFTSettings::zero_poly_via_multiplication(self, missing_idxs, domain_size);
        Ok((zero_eval, Polynomial::from_fr(zero_poly)))
    }
}


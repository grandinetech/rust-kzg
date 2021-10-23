use crate::data_types::fr::Fr;
use crate::fk20_fft::FFTSettings;
use crate::kzg10::Polynomial;
use kzg::ZeroPoly;

impl ZeroPoly<Fr, Polynomial> for FFTSettings {
    fn do_zero_poly_mul_partial(&self, idxs: &[usize], stride: usize) -> Result<Polynomial, String> {
        todo!()
    }

    fn reduce_partials(&self, domain_size: usize, partials: &[Polynomial]) -> Result<Polynomial, String> {
       todo!()
    }
    
    fn zero_poly_via_multiplication(&self, domain_size: usize, missing_idxs: &[usize]) -> Result<(Vec<Fr>, Polynomial), String> {
        let (zero_eval, zero_poly) = FFTSettings::zero_poly_via_multiplication(self, missing_idxs, domain_size);
        Ok((zero_eval, Polynomial::from_fr(zero_poly)))
    }
}


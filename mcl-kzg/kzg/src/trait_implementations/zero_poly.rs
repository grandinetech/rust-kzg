use crate::data_types::fr::Fr;
use crate::fk20_fft::FFTSettings;
use crate::kzg10::Polynomial;
use kzg::ZeroPoly;

impl ZeroPoly<Fr, Polynomial> for FFTSettings {
    fn do_zero_poly_mul_partial(&self, idxs: &[usize], stride: usize) -> Result<Polynomial, String> {
        self.do_zero_poly_mul_partial(idxs, stride)
    }

    fn reduce_partials(&self, domain_size: usize, partials: &[Polynomial]) -> Result<Polynomial, String> {
       self.reduce_partials(domain_size, partials)
    }

    fn zero_poly_via_multiplication(&self, domain_size: usize, missing_idxs: &[usize]) -> Result<(Vec<Fr>, Polynomial), String> {
        self.zero_poly_via_multiplication(domain_size, missing_idxs)
    }
}

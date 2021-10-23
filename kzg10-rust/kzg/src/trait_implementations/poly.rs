use crate::data_types::fr::Fr;
use kzg::Poly;
use crate::kzg10::Polynomial;

impl Poly<Fr> for Polynomial {
    fn default() -> Result<Self, String> {
        Ok(Polynomial { coeffs: vec![] })
    }

    fn new(size: usize) -> Result<Self, String> {
        Ok(Polynomial {
            coeffs: vec![Fr::default(); size],
        })
    }

    fn get_coeff_at(&self, i: usize) -> Fr {
        self.coeffs[i]
        // self.coeffs.get(i)
    }

    fn set_coeff_at(&mut self, i: usize, x: &Fr) {
        self.coeffs[i] = x.clone();
    }

    fn get_coeffs(&self) -> &[Fr] {
        &self.coeffs
    }

    fn len(&self) -> usize {
        Polynomial::order(self)
    }

    fn eval(&self, x: &Fr) -> Fr {
        Polynomial::eval_at(self, x)
    }

    fn scale(&mut self) {
        todo!()
    }

    fn unscale(&mut self) {
        todo!()
    }

    fn inverse(&mut self, new_len: usize) -> Result<Self, String> {
        todo!()
    }

    fn div(&mut self, x: &Self) -> Result<Self, String> {
        todo!()
    }

    fn destroy(&mut self) {
        todo!()
    }
}

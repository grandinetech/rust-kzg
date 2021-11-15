use crate::data_types::fr::Fr;
use crate::kzg10::Polynomial;
use kzg::Poly;
use kzg::FFTSettingsPoly;
use kzg::PolyRecover;
use crate::fk20_fft::FFTSettings;

impl Poly<Fr> for Polynomial {
    fn default() -> Self {
        Polynomial { coeffs: vec![] }
    }

    fn new(size: usize) -> Result<Self, String> {
        Ok(Polynomial::new(size))
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
        Polynomial::inverse(self, new_len)
    }

    fn div(&mut self, x: &Self) -> Result<Self, String> { 
        Polynomial::div(self, &x.coeffs)
    }

    fn long_div(&mut self, x: &Self) -> Result<Self, String> {
        Polynomial::long_division(self, &x.coeffs)
    }

    fn fast_div(&mut self, x: &Self) -> Result<Self, String> {
        Polynomial::fast_div(self, &x.coeffs)
    }

    fn mul_direct(&mut self, x: &Self, len: usize) -> Result<Self, String> {
        Polynomial::mul_direct(self, x, len)
    }

    fn destroy(&mut self) {
    }
}

impl FFTSettingsPoly<Fr, Polynomial, FFTSettings> for FFTSettings {
    fn poly_mul_fft(a: &Polynomial, x: &Polynomial, len: usize, fs: Option<&FFTSettings>) -> Result<Polynomial, String> {
        // Polynomial::mul(a, x, _fs, len)
        Polynomial::mul_fft(a, x, fs, len)
    }
}

impl PolyRecover<Fr, Polynomial, FFTSettings> for Polynomial {
    fn recover_poly_from_samples(samples: &[Option<Fr>], fs: FFTSettings) -> Self {
        Polynomial::recover_from_samples(fs, &samples)
    }
}

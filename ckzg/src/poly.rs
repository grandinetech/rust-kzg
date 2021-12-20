use std::slice;
use kzg::{Fr, Poly, PolyRecover};
use crate::finite::BlstFr;
use crate::consts::KzgRet;
use crate::fftsettings::KzgFFTSettings;
use crate::RUN_PARALLEL;

extern "C" {
    fn new_poly(out: *mut KzgPoly, length: u64) -> KzgRet;
    fn free_poly(p: *mut KzgPoly);
    fn new_poly_div(out: *mut KzgPoly, dividend: *const KzgPoly, divisor: *const KzgPoly) -> KzgRet;
    fn eval_poly(out: *mut BlstFr, p: *const KzgPoly, x: *const BlstFr);
    fn poly_inverse(out: *mut KzgPoly, b: *mut KzgPoly) -> KzgRet;
    fn poly_mul(out: *mut KzgPoly, a: *const KzgPoly, b: *const KzgPoly, run_parallel: bool) -> KzgRet;
    fn poly_long_div(out: *mut KzgPoly, dividend: *const KzgPoly, divisor: *const KzgPoly) -> KzgRet;
    fn poly_fast_div(out: *mut KzgPoly, dividend: *const KzgPoly, divisor: *const KzgPoly) -> KzgRet;
    fn recover_poly_from_samples(reconstructed_data: *mut BlstFr, samples: *mut BlstFr, len_samples: u64, fs: *const KzgFFTSettings, run_parallel: bool) -> KzgRet;
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct KzgPoly {
    pub coeffs: *mut BlstFr,
    pub length: u64
}

impl Poly<BlstFr> for KzgPoly {
    fn default() -> Self {
        Self {
            coeffs: &mut Fr::default(),
            length: 0
        }
    }

    fn new(size: usize) -> Result<Self, String> {
        let mut poly = Poly::default();
        unsafe {
            match new_poly(&mut poly, size as u64) {
                KzgRet::KzgOk => Ok(poly),
                e => Err(format!("An error has occurred in Poly::new ==> {:?}", e))
            }
        }
    }

    fn get_coeff_at(&self, i: usize) -> BlstFr {
        unsafe {
            *self.coeffs.add(i) as BlstFr
        }
    }

    fn set_coeff_at(&mut self, i: usize, x: &BlstFr) {
        unsafe {
            *self.coeffs.add(i) = *x;
        }
    }

    fn get_coeffs(&self) -> &[BlstFr] {
        unsafe {
            slice::from_raw_parts(self.coeffs, self.length as usize)
        }
    }

    fn len(&self) -> usize {
        self.length as usize
    }

    fn eval(&self, x: &BlstFr) -> BlstFr {
        let mut out = Fr::default();
        unsafe {
            eval_poly(&mut out, self, x);
        }
        out
    }

    fn scale(&mut self) {
        todo!()
    }

    fn unscale(&mut self) {
        todo!()
    }

    fn inverse(&mut self, new_len: usize) -> Result<Self, String> {
        let mut poly = KzgPoly::new(new_len).unwrap();
        unsafe {
            match poly_inverse(&mut poly, self) {
                KzgRet::KzgOk => Ok(poly),
                e => Err(format!("An error has occurred in Poly::inverse ==> {:?}", e))
            }
        }
    }

    fn div(&mut self, x: &Self) -> Result<Self, String> {
        let mut poly = Poly::default();
        unsafe {
            match new_poly_div(&mut poly, self, x) {
                KzgRet::KzgOk => Ok(poly),
                e => Err(format!("An error has occurred in Poly::div ==> {:?}", e))
            }
        }
    }

    fn long_div(&mut self, x: &Self) -> Result<Self, String> {
        let mut poly = Poly::new(self.len()).unwrap();
        unsafe {
            match poly_long_div(&mut poly, self, x) {
                KzgRet::KzgOk => Ok(poly),
                e => Err(format!("An error has occurred in Poly::long_div ==> {:?}", e))
            }
        }
    }

    fn fast_div(&mut self, x: &Self) -> Result<Self, String> {
        let mut poly = Poly::new(self.len()).unwrap();
        unsafe {
            match poly_fast_div(&mut poly, self, x) {
                KzgRet::KzgOk => Ok(poly),
                e => Err(format!("An error has occurred in Poly::fast_div ==> {:?}", e))
            }
        }
    }

    fn mul_direct(&mut self, x: &Self, len: usize) -> Result<Self, String> {
        let mut poly = Poly::new(len).unwrap();
        unsafe {
            match poly_mul(&mut poly, self, x, RUN_PARALLEL) {
                KzgRet::KzgOk => Ok(poly),
                e => Err(format!("An error has occurred in Poly::mul_direct ==> {:?}", e))
            }
        }
    }
}

impl Drop for KzgPoly {
    fn drop(&mut self) {
        unsafe {
            free_poly(self);
        }
    }
}

impl PolyRecover<BlstFr, KzgPoly, KzgFFTSettings> for KzgPoly {
    fn recover_poly_from_samples(samples: &[Option<BlstFr>], fs: &KzgFFTSettings) -> Result<KzgPoly, String> {
        let mut reconstructed_data = vec![Fr::default(); samples.len()];
        let mut optionless_samples = Vec::new();
        for s in samples {
            if s.is_some() {
                optionless_samples.push(s.unwrap());
                continue
            }
            optionless_samples.push(Fr::null());
        }
        unsafe {
            match recover_poly_from_samples(reconstructed_data.as_mut_ptr(), optionless_samples.as_mut_ptr(), samples.len() as u64, fs, RUN_PARALLEL) {
                KzgRet::KzgOk => (),
                e => return Err(format!("An error has occurred in PolyRecover::recover_poly_from_samples ==> {:?}", e))
            }
        }
        let mut out = KzgPoly::new(reconstructed_data.len()).unwrap();
        for (i, data) in reconstructed_data.iter().enumerate() {
            out.set_coeff_at(i, data)
        }
        Ok(out)
    }
}

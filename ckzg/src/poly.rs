use kzg::{Fr, Poly};
use crate::finite::BlstFr;
use crate::common::KzgRet;

extern "C" {
    fn new_poly(out: *mut KzgPoly, length: u64) -> KzgRet;
    fn free_poly(p: *mut KzgPoly);
    fn new_poly_div(out: *mut KzgPoly, dividend: *const KzgPoly, divisor: *const KzgPoly) -> KzgRet;
    fn eval_poly(out: *mut BlstFr, p: *const KzgPoly, x: *const BlstFr);
    fn poly_inverse(out: *mut KzgPoly, b: *mut KzgPoly) -> KzgRet;
    fn poly_mul(out: *mut KzgPoly, a: *const KzgPoly, b: *const KzgPoly) -> KzgRet;
    fn poly_long_div(out: *mut KzgPoly, dividend: *const KzgPoly, divisor: *const KzgPoly) -> KzgRet;
    fn poly_fast_div(out: *mut KzgPoly, dividend: *const KzgPoly, divisor: *const KzgPoly) -> KzgRet;
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct KzgPoly {
    pub coeffs: *mut BlstFr,
    pub length: u64
}

impl Poly<BlstFr> for KzgPoly {
    fn default() -> Self {
        Self {
            coeffs: &mut Fr::default(),
            length: 4
        }
    }

    fn new(size: usize) -> Result<Self, String> {
        let mut poly = Poly::default();
        unsafe {
            return match new_poly(&mut poly, size as u64) {
                KzgRet::KzgOk => Ok(poly),
                e => Err(format!("An error has occurred in Poly::new ==> {:?}", e))
            }
        }
    }

    fn get_coeff_at(&self, i: usize) -> BlstFr {
        unsafe {
            return *self.coeffs.offset(i as isize) as BlstFr;
        }
    }

    fn set_coeff_at(&mut self, i: usize, x: &BlstFr) {
        unsafe {
            *self.coeffs.offset(i as isize) = *x;
        }
    }

    fn get_coeffs(&self) -> &[BlstFr] {
        todo!()
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
            return match poly_inverse(&mut poly, self) {
                KzgRet::KzgOk => {
                    self.destroy();
                    Ok(poly)
                },
                e => Err(format!("An error has occurred in Poly::inverse ==> {:?}", e))
            }
        }
    }

    fn div(&mut self, x: &Self) -> Result<Self, String> {
        let mut poly = Poly::default();
        unsafe {
            return match new_poly_div(&mut poly, self, x) {
                KzgRet::KzgOk => Ok(poly),
                e => Err(format!("An error has occurred in Poly::div ==> {:?}", e))
            }
        }
    }

    fn long_div(&mut self, x: &Self) -> Result<Self, String> {
        let mut poly = Poly::new(self.len()).unwrap();
        unsafe {
            return match poly_long_div(&mut poly, self, x) {
                KzgRet::KzgOk => Ok(poly),
                e => Err(format!("An error has occurred in Poly::long_div ==> {:?}", e))
            }
        }
    }

    fn fast_div(&mut self, x: &Self) -> Result<Self, String> {
        let mut poly = Poly::new(self.len()).unwrap();
        unsafe {
            return match poly_fast_div(&mut poly, self, x) {
                KzgRet::KzgOk => Ok(poly),
                e => Err(format!("An error has occurred in Poly::fast_div ==> {:?}", e))
            }
        }
    }

    fn mul_direct(&mut self, x: &Self, len: usize) -> Result<Self, String> {
        let mut poly = Poly::new(len).unwrap();
        unsafe {
            return match poly_mul(&mut poly, self, x) {
                KzgRet::KzgOk => Ok(poly),
                e => Err(format!("An error has occurred in Poly::mul_direct ==> {:?}", e))
            }
        }
    }

    fn destroy(&mut self) {
        unsafe {
            free_poly(self);
        }
    }
}

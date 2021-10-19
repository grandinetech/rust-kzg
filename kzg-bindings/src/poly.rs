use kzg::{Fr, Poly};
use crate::finite::BlstFr;
use crate::common::KzgRet;

#[link(name = "ckzg", kind = "static")]
extern "C" {
    fn new_poly(out: *mut KzgPoly, length: u64) -> KzgRet;
    fn free_poly(p: *mut KzgPoly);
    fn new_poly_div(out: *mut KzgPoly, dividend: *const KzgPoly, divisor: *const KzgPoly) -> KzgRet;
    fn eval_poly(out: *mut BlstFr, p: *const KzgPoly, x: *const BlstFr);
    fn poly_inverse(out: *mut KzgPoly, b: *mut KzgPoly) -> KzgRet;
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

    fn new(size: u64) -> Result<Self, String> where Self: Sized {
        let mut poly = Poly::default();
        unsafe {
            return match new_poly(&mut poly, size) {
                KzgRet::KzgOk => Ok(
                    Self {
                        coeffs: poly.coeffs,
                        length: poly.length
                    }
                ),
                e => Err(format!("An error has occurred in \"Poly::new\" ==> {:?}", e))
            }
        }
    }

    fn get_coeff_at(&self, i: u64) -> BlstFr where BlstFr: Sized {
        unsafe {
            return *self.coeffs.offset(i as isize) as BlstFr;
        }
    }

    fn set_coeff_at(&mut self, i: u64, x: &BlstFr) {
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

    fn inverse(&mut self, p: *mut Self) -> Result<(), String> where Self: Sized {
        unsafe {
            return match poly_inverse(self, p) {
                KzgRet::KzgOk => Ok(()),
                e => Err(format!("An error has occurred in \"Poly::inverse\" ==> {:?}", e))
            }
        }
    }

    fn destroy(&mut self) {
        unsafe {
            free_poly(self);
        }
    }

    /*
    pub fn new_divided(dividend: *const Poly, divisor: *const Poly) -> Result<Poly, Error> {
        let mut poly = Poly::default();
        unsafe {
            return match new_poly_div(&mut poly, dividend, divisor) {
                Error::KzgOk => Ok(Poly::_self(&mut poly)),
                e => {
                    println!("An error has occurred in \"Poly::new_divided\" ==> {:?}", e);
                    Err(e)
                }
            }
        }
    }

    // https://github.com/benjaminion/c-kzg/blob/63612c11192cea02b2cb78aa677f570041b6b763/src/poly_bench.c#L39
    fn randomize_coeffs(&mut self) {
        for i in 0..self.length {
            self.set_coeff_at(i, Fr::rand());
        }
    }

    // Ensure that the polynomials' orders corresponds to their lengths
    // https://github.com/benjaminion/c-kzg/blob/63612c11192cea02b2cb78aa677f570041b6b763/src/poly_bench.c#L46
    fn check_order(&mut self) {
        if self.get_coeff_at(self.length - 1).is_zero() {
            self.set_coeff_at(self.length - 1, Fr::one());
        }
    }

    // Used only for benchmarks
    pub fn bench_divide_in_finite_field(scale: u64) -> Error {
        let dividend_length: u64 = 1 << scale;
        let divisor_length: u64 = dividend_length / 2;

        let mut dividend = match Poly::new(dividend_length) {
            Ok(p) => p,
            Err(_) => Poly::default()
        };
        let mut divisor = match Poly::new(divisor_length) {
            Ok(p) => p,
            Err(_) => Poly::default()
        };

        dividend.randomize_coeffs();
        divisor.randomize_coeffs();
        dividend.check_order();
        divisor.check_order();

        let mut errors = Error::KzgOk;
        let mut divided_poly = match Poly::new_divided(&mut dividend, &mut divisor) {
            Ok(p) => p,
            Err(e) => {
                errors = e;
                Poly::default()
            }
        };

        dividend.destroy();
        divisor.destroy();
        divided_poly.destroy();

        errors
    }
    */
}

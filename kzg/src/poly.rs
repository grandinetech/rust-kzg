use super::{Error, Fr, Poly};

#[link(name = "ckzg", kind = "static")]
extern "C" {
    fn new_poly(out: *mut Poly, length: u64) -> Error;
    fn free_poly(p: *mut Poly);
    fn new_poly_div(out: *mut Poly, dividend: *const Poly, divisor: *const Poly) -> Error;
    fn eval_poly(out: *mut Fr, p: *const Poly, x: *const Fr);
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct KzgPoly {
    pub coeffs: *mut Fr,
    pub length: u64
}

impl Poly {
    pub fn default() -> Self {
        Self { coeffs: &mut Fr { l: [0, 0, 0, 0] }, length: 4 }
    }

    pub fn new(length: u64) -> Result<Self, Error> {
        let mut poly = Poly::default();
        unsafe {
            return match new_poly(&mut poly, length) {
                Error::KzgOk => {
                    Ok(Self {
                        coeffs: poly.coeffs,
                        length: poly.length
                    })
                }
                e => {
                    println!("An error has occurred in \"Poly::new\" : {:?}", e);
                    Err(e)
                }
            }
        }
    }

    pub fn new_divided(dividend: *const Poly, divisor: *const Poly) -> Result<Poly, Error> {
        let mut poly = Poly::default();
        unsafe {
            return match new_poly_div(&mut poly, dividend, divisor) {
                Error::KzgOk => {
                    Ok(Self {
                        coeffs: poly.coeffs,
                        length: poly.length
                    })
                }
                e => {
                    println!("An error has occurred in \"Poly::new_divided\" : {:?}", e);
                    Err(e)
                }
            }
        }
    }

    // https://github.com/benjaminion/c-kzg/blob/63612c11192cea02b2cb78aa677f570041b6b763/src/poly_bench.c#L39
    fn randomize_coeffs(&mut self, length: u64) {
        for i in 0..length {
            self.set_coeff_at(i as isize, Fr::rand());
        }
    }

    // Ensure that the polynomials' orders corresponds to their lengths
    // https://github.com/benjaminion/c-kzg/blob/63612c11192cea02b2cb78aa677f570041b6b763/src/poly_bench.c#L46
    fn check_order(&mut self, length: u64) {
        if self.get_coeff_at((length - 1) as isize).is_zero() {
            self.set_coeff_at((length - 1) as isize, Fr::one());
        }
    }

    pub fn divide_in_finite_field(scale: u64) -> Error {
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

        dividend.randomize_coeffs(dividend_length);
        divisor.randomize_coeffs(divisor_length);

        dividend.check_order(dividend_length);
        divisor.check_order(divisor_length);

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

    pub fn eval_at(&self, point: &Fr) -> Fr {
        let mut out = Fr::default();
        unsafe {
            eval_poly(&mut out, self, point);
        }
        out
    }

    pub fn get_coeff_at(self, index: isize) -> Fr {
        unsafe {
            return *self.coeffs.offset(index as isize) as Fr;
        }
    }

    pub fn set_coeff_at(&mut self, index: isize, point: Fr) {
        unsafe {
            *self.coeffs.offset(index) = point;
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            free_poly(self);
        }
    }
}

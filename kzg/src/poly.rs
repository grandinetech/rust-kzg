use super::finite::{rand_fr, is_zero_fr, u64_to_fr};
use super::Poly;
use super::Error;

#[link(name = "ckzg", kind = "static")]
extern "C" {
    fn new_poly(out: *mut Poly, length: u64) -> Error;
    fn free_poly(p: *mut Poly);
    fn new_poly_div(out: *mut Poly, dividend: *const Poly, divisor: *const Poly) -> Error;
}

pub fn create_poly(length: u64) -> Result<Poly, Error> {
    let mut poly = Poly::default();
    unsafe {
        let error = new_poly(&mut poly, length);
        return match error {
            Error::KzgOk => Ok(poly),
            _ => Err(error)
        }
    }
}

pub fn destroy_poly(poly: &mut Poly) {
    unsafe {
        free_poly(poly);
    }
}

pub fn create_divided_poly(dividend: *const Poly, divisor: *const Poly) -> Result<Poly, Error> {
    let mut poly = Poly::default();
    unsafe {
        let error = new_poly_div(&mut poly, dividend, divisor);
        return match error {
            Error::KzgOk => Ok(poly),
            _ => Err(error)
        }
    }
}

// https://github.com/benjaminion/c-kzg/blob/63612c11192cea02b2cb78aa677f570041b6b763/src/poly_bench.c#L39
fn randomize_poly_coefficients(poly: &mut Poly, length: u64) {
    for i in 0..length  {
        unsafe {
            *poly.coeffs.offset(i as isize) = rand_fr();
        }
    }
}

// Ensure that the polynomials' orders corresponds to their lengths
// https://github.com/benjaminion/c-kzg/blob/63612c11192cea02b2cb78aa677f570041b6b763/src/poly_bench.c#L46
fn check_poly_order(poly: &mut Poly, length: u64) {
    unsafe {
        if is_zero_fr(*poly.coeffs.offset((length - 1) as isize)) {
            *poly.coeffs.offset((length - 1) as isize) = u64_to_fr(1);
        }
    }
}

pub fn poly_division_in_finite_field(scale : u64) -> Error {
    let dividend_length: u64 = 1 << scale;
    let divisor_length: u64 = dividend_length / 2;

    let mut dividend = match create_poly(dividend_length) {
        Ok(p) => p,
        Err(_) => Poly::default()
    };
    let mut divisor = match create_poly(divisor_length) {
        Ok(p) => p,
        Err(_) => Poly::default()
    };

    randomize_poly_coefficients(&mut dividend, dividend_length);
    randomize_poly_coefficients(&mut divisor, divisor_length);

    check_poly_order(&mut dividend, dividend_length);
    check_poly_order(&mut divisor, divisor_length);

    let mut errors = Error::KzgOk;
    let mut divided_poly = match create_divided_poly(&mut dividend, &mut divisor) {
        Ok(p) => p,
        Err(e) => {
            errors = e;
            Poly::default()
        }
    };

    destroy_poly(&mut dividend);
    destroy_poly(&mut divisor);
    destroy_poly(&mut divided_poly);

    errors
}

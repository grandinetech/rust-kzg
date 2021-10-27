/*
use crate::kzg_types::{FFTSettings, Poly};
use blst::{
    blst_fr_eucl_inverse,
    blst_fr_mul,
    blst_fr_sub,
    blst_fr_add,
};
use kzg::IFr;
use crate::kzg_types::Fr;
use crate::utils::{next_power_of_two, log2_pow2, log2_u64, min_u64};
use crate::fft_fr::fft_fr;

pub fn fr_div(a: &Fr, b: &Fr) -> Result<Fr, String> {
    let mut tmp = Fr::default();
    let mut out = Fr::default();

    unsafe {
        blst_fr_eucl_inverse(&mut tmp, b);
        blst_fr_mul(&mut out, a, &tmp);
    }
    Ok(out)
}

pub fn poly_norm(p: &Poly) -> Result<Poly, String> {
    let mut ret = p.clone();
    let mut temp_len: usize = ret.coeffs.len();
    while temp_len > 0 && &ret.coeffs[temp_len - 1].is_zero() {
        temp_len -= 1;
    }
    if temp_len == 0 {
        ret.coeffs = Vec::default();
    }

    Ok(ret)
}

pub fn poly_quotient_length(dividend: &Poly, divisor: &Poly) -> Result<usize, String> {
    if dividend.coeffs.len() >= divisor.coeffs.len() {
        return Ok(dividend.coeffs.len() - divisor.coeffs.len() + 1);
    }

    Ok(0)
}

pub fn poly_long_div(dividend: &Poly, divisor: &Poly) -> Result<Poly, String> {
    // Dividing by zero is undefined
    assert!(divisor.coeffs.len() > 0);
    // The divisor's highest coefficient must be non-zero
    //CHECK(!fr_is_zero(&divisor->coeffs[divisor->length - 1]));
    assert!(!divisor.coeffs[divisor.coeffs.len() - 1].is_zero());

    let mut out: Poly = Poly { coeffs: Vec::default() };
    //uint64_t a_pos = dividend->length - 1;
    //uint64_t b_pos = divisor->length - 1;
    //uint64_t diff = a_pos - b_pos;
    let mut a_pos = dividend.coeffs.len();
    let b_pos = divisor.coeffs.len();
    let mut diff = a_pos - b_pos;

    // Deal with the size of the output polynomial
    // uint64_t out_length = poly_quotient_length(dividend, divisor);
    let result = poly_quotient_length(&dividend, &divisor);
    assert!(result.is_ok());
    let out_length = result.unwrap();

    // CHECK(out->length >= out_length);
    assert!(out.coeffs.len() >= out_length);

    // If the divisor is larger than the dividend, the result is zero-length
    if out_length == 0 {
        return Ok(out);
    }

    //fr_t *a;
    // TRY(new_fr_array(&a, dividend->length));
    let mut a = vec![Fr::default(); dividend.coeffs.len()];
    for i in 0..dividend.coeffs.len() {
        //a[i] = dividend->coeffs[i];
        a.push(dividend.coeffs[i]);
    }

    while diff > 0 {
        // fr_div(&out->coeffs[diff], &a[a_pos], &divisor->coeffs[b_pos]);
        let result = fr_div(&a[a_pos], &divisor.coeffs[b_pos]);
        assert!(result.is_ok());
        out.coeffs[diff] = result.unwrap();

        unsafe {
            for i in 0..(b_pos + 1) {
                // fr_t tmp;
                let mut tmp = Fr::default();
                // a[diff + i] -= b[i] * quot
                blst_fr_mul(&mut tmp, &out.coeffs[diff], &divisor.coeffs[i]);
                blst_fr_sub(&mut a[diff + i], &a[diff + i], &tmp);
            }
        }
        diff -= 1;
        a_pos -= 1;
    }
    // fr_div(&out->coeffs[0], &a[a_pos], &divisor->coeffs[b_pos]);
    let result = fr_div(&a[a_pos], &divisor.coeffs[b_pos]);
    assert!(result.is_ok());
    out.coeffs[0] = result.unwrap();

    Ok(out)
}

pub fn poly_flip(input: &Poly) -> Result<Poly, String> {
    let mut output = Poly { coeffs: Vec::default() };
    for i in 0..input.coeffs.len() {
        output.coeffs.push(input.coeffs[input.coeffs.len() - i - 1]);
    }
    Ok(output)
}

pub fn poly_fast_div(dividend: &Poly, divisor: &Poly) -> Result<Poly, String> {

    // Dividing by zero is undefined
    assert!(divisor.coeffs.len() > 0);

    // The divisor's highest coefficient must be non-zero
    assert!(!&divisor.coeffs[divisor.coeffs.len() - 1].is_zero());

    let m: usize = dividend.coeffs.len() - 1;
    let n: usize = divisor.coeffs.len() - 1;

    // If the divisor is larger than the dividend, the result is zero-length
    if n > m {
        return Ok(Poly { coeffs: Vec::default() });
    }

    // Ensure the output poly has enough space allocated
    //CHECK(out->length >= m - n + 1);

    // Ensure that the divisor is well-formed for the inverse operation
    assert!(!&divisor.coeffs[divisor.coeffs.len() - 1].is_zero());

    let mut out = Poly { coeffs: Vec::default() };
    // Special case for divisor.length == 1 (it's a constant)
    if divisor.coeffs.len() == 1 {
        //out->length = dividend->length;
        for i in 0..dividend.coeffs.len() {
            out.coeffs.push(fr_div(&dividend.coeffs[i], &divisor.coeffs[0]).unwrap());
        }
        return Ok(out);
    }

    // poly a_flip, b_flip;
    let mut a_flip = Poly { coeffs: Vec::default() };
    let mut b_flip = Poly { coeffs: Vec::default() };

    // TRY(new_poly(&a_flip, dividend->length));
    // TRY(new_poly(&b_flip, divisor->length));
    // TRY(poly_flip(&a_flip, dividend));
    // TRY(poly_flip(&b_flip, divisor));
    a_flip = poly_flip(&dividend).unwrap();
    b_flip = poly_flip(&divisor).unwrap();

    // poly inv_b_flip;
    let mut inv_b_flip = Poly { coeffs: Vec::default() };
    // TRY(new_poly(&inv_b_flip, m - n + 1));
    // TRY(poly_inverse(&inv_b_flip, &b_flip));
    inv_b_flip = poly_inverse(&b_flip, m - n + 1).unwrap();

    // poly q_flip;
    let mut q_flip = Poly { coeffs: Vec::default() };
    // We need only m - n + 1 coefficients of q_flip
    // TRY(new_poly(&q_flip, m - n + 1));
    // TRY(poly_mul(&q_flip, &a_flip, &inv_b_flip));

    q_flip = poly_mul(&a_flip, &inv_b_flip, m - n + 1).unwrap();

    // out->length = m - n + 1;
    // TRY(poly_flip(out, &q_flip));
    out = poly_flip(&q_flip).unwrap();

    Ok(out)
}

pub fn new_poly_div(dividend_: &Poly, divisor_: &Poly) -> Result<Poly, String> {

    //poly dividend = poly_norm(dividend_);
    let result = poly_norm(dividend_);
    assert!(result.is_ok());
    let dividend: Poly = result.unwrap();

    //poly divisor = poly_norm(divisor_);
    let result = poly_norm(divisor_);
    assert!(result.is_ok());
    let divisor: Poly = result.unwrap();

    //TRY(new_poly(out, poly_quotient_length(&dividend, &divisor)));
    // let newLength = poly_quotient_length(&dividend, &divisor).unwrap();

    if divisor.coeffs.len() >= dividend.coeffs.len() || divisor.coeffs.len() < 128 { // Tunable paramter
        let result = poly_long_div(&dividend, &divisor);
        assert!(result.is_ok());
        result
    } else {
        let result = poly_fast_div(&dividend, &divisor);
        assert!(result.is_ok());
        result
    }
}
 */

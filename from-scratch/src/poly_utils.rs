use kzg::{Fr};
use crate::kzg_types::FsPoly;

// TODO: probably insert into poly trait
pub fn poly_norm(p: &FsPoly) -> Result<FsPoly, String> {
    let mut ret = p.clone();
    let mut temp_len: usize = ret.coeffs.len();
    while temp_len > 0 && ret.coeffs[temp_len - 1].is_zero() {
        temp_len -= 1;
    }
    if temp_len == 0 {
        ret.coeffs = Vec::default();
    }

    Ok(ret)
}

// TODO: probably insert into poly trait
pub fn poly_quotient_length(dividend: &FsPoly, divisor: &FsPoly) -> Result<usize, String> {
    if dividend.coeffs.len() >= divisor.coeffs.len() {
        return Ok(dividend.coeffs.len() - divisor.coeffs.len() + 1);
    }

    Ok(0)
}

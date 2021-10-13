use crate::fft_fr::fft_fr;
use crate::kzg_types::{create_fr_one, create_fr_zero, negate_fr, FFTSettings, Poly};
use crate::utils::is_power_of_two;
use blst::{blst_fr_add, blst_fr_mul};
use kzg::Fr;

pub fn do_zero_poly_mul_partial(poly: &mut Poly, idxs: &[usize], stride: &usize, fft_settings: &FFTSettings) -> Result<(), String> {
    if idxs.len() == 0 {
        return Err(String::from("idx array must be non-zero"));
    } else if poly.coeffs.len() < idxs.len() + 1 {
        return Err(String::from("idx array must be non-zero"));
    }

    negate_fr(&mut poly.coeffs[0], &fft_settings.expanded_roots_of_unity[idxs[0] * stride]);

    for i in 1..idxs.len() {
        let mut neg_di: Fr = Fr::default();
        negate_fr(&mut neg_di, &fft_settings.expanded_roots_of_unity[idxs[i] * stride]);
        poly.coeffs[i] = neg_di.clone();
        unsafe {
            blst_fr_add(&mut poly.coeffs[i], &poly.coeffs[i], &poly.coeffs[i - 1]);
        }

        let mut j = i - 1;
        while j > 0 {
            unsafe {
                blst_fr_mul(&mut poly.coeffs[j], &poly.coeffs[j], &neg_di);
                blst_fr_add(&mut poly.coeffs[j], &poly.coeffs[j], &poly.coeffs[j - 1]);
            }
            j -= 1;
        }
        unsafe {
            blst_fr_mul(&mut poly.coeffs[0], &poly.coeffs[0], &neg_di);
        }
    }

    poly.coeffs[idxs.len()] = create_fr_one();
    for i in (idxs.len() + 1)..poly.coeffs.len() {
        poly.coeffs[i] = create_fr_zero();
    }

    return Ok(());
}

pub fn pad_poly(ret: &mut [Fr], poly: &Poly) -> Result<(), String> {
    if ret.len() < poly.coeffs.len() {
        return Err(String::from("Expected return array to be as lengthy as provided polynomial"));
    }

    for i in 0..poly.coeffs.len() {
        ret[i] = poly.coeffs[i].clone();
    }

    for i in poly.coeffs.len()..ret.len() {
        ret[i] = create_fr_zero();
    }

    return Ok(());
}

pub fn reduce_partials(ret: &mut Poly, scratch: &[Fr], partials: &[Poly], fft_settings: &FFTSettings) -> Result<(), String> {
    if !is_power_of_two(ret.coeffs.len()) {
        return Err(String::from("Expected poly needs to be a power of two length"));
    } else if scratch.len() < 3 * ret.coeffs.len() {
        return Err(String::from("Expected scratch length to be at least 3 times the polynomial buffer"));
    }

    let mut out_degree: usize = 0;
    for i in 0..partials.len() {
        out_degree += partials[i].coeffs.len() - 1;
    }

    if out_degree + 1 > ret.coeffs.len() {
        return Err(String::from("Out degree not expected to be more than return polynomial length"));
    }

    let mut p_padded = scratch[..ret.coeffs.len()].to_vec();
    let mut mul_eval_ps = scratch[ret.coeffs.len()..(ret.coeffs.len() * 2)].to_vec();
    let mut p_eval = scratch[(ret.coeffs.len() * 2)..].to_vec();

    pad_poly(&mut p_padded, &partials[partials.len() - 1])?;
    fft_fr(&mut mul_eval_ps, &p_padded, false, fft_settings)?;

    for i in 0..(partials.len() - 1) {
        pad_poly(&mut p_padded, &partials[i])?;
        fft_fr(&mut p_eval, &p_padded, false, fft_settings)?;
        for j in 0..ret.coeffs.len() {
            unsafe {
                blst_fr_mul(&mut mul_eval_ps[j], &mul_eval_ps[j], &p_eval[j]);
            }
        }
    }

    fft_fr(&mut ret.coeffs, &mul_eval_ps, true, fft_settings)?;

    return Ok(());
}


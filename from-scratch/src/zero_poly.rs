use std::cmp::min;
use crate::fft_fr::fft_fr;
use crate::kzg_types::{create_fr_one, create_fr_zero, negate_fr, FFTSettings, Poly};
use crate::utils::{is_power_of_two, next_power_of_two};
use blst::{blst_fr_add, blst_fr_mul};
use kzg::Fr;

pub fn do_zero_poly_mul_partial(poly: &mut Poly, idxs: &[usize], stride: usize, fft_settings: &FFTSettings) -> Result<(), String> {
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

    poly.coeffs = poly.coeffs[..idxs.len() + 1].to_vec();

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

pub fn zero_polynomial_via_multiplication(zero_eval: &mut [Fr], zero_poly: &mut Poly, missing_idxs: &[usize], fft_settings: &FFTSettings) -> Result<(), String> {
    if zero_eval.len() != zero_poly.coeffs.len() {
        return Err(String::from("Zero eval must be same length as zero_poly.coeffs"));
    }

    if missing_idxs.len() == 0 {
        zero_poly.coeffs = Vec::new();
        for i in 0..zero_eval.len() {
            zero_eval[i] = create_fr_zero();
        }

        return Ok(());
    }

    if missing_idxs.len() >= zero_eval.len() {
        return Err(String::from("Missing idxs mustn't be longer than zero_eval"));
    } else if zero_eval.len() > fft_settings.max_width {
        return Err(String::from("Zero eval length must be less or equal to fft_settings.max_width"));
    } else if !is_power_of_two(zero_eval.len()) {
        return Err(String::from("Zero eval length must be a power of 2"));
    }

    let degree_of_partial = 64;
    let missing_per_partial = degree_of_partial - 1;
    let domain_stride = fft_settings.max_width / zero_eval.len();
    let mut partial_count = (missing_idxs.len() + missing_per_partial - 1) / missing_per_partial;
    let n = min(next_power_of_two(partial_count * degree_of_partial), zero_eval.len());

    if missing_idxs.len() <= missing_per_partial {
        do_zero_poly_mul_partial(zero_poly, &missing_idxs, domain_stride, &fft_settings)?;
        fft_fr(&mut zero_eval[..zero_poly.coeffs.len()], &zero_poly.coeffs, false, fft_settings)?;
    } else {
        let work = vec![Fr::default(); next_power_of_two(partial_count * degree_of_partial)];
        let mut partials = Vec::new();
        let mut offset = 0;
        let mut out_offset = 0;
        let mut max = missing_idxs.len();
        for i in 0..partial_count {
            let end = min(offset + missing_per_partial, max);
            partials.push(Poly { coeffs: work[out_offset..(out_offset + degree_of_partial)].to_vec() });
            do_zero_poly_mul_partial(&mut partials[i], &missing_idxs[offset..end], domain_stride, fft_settings)?;
            offset += missing_per_partial;
            out_offset += degree_of_partial;
        }
        partials[partial_count - 1].coeffs = partials[partial_count - 1].coeffs[..(1 + missing_idxs.len() - (partial_count - 1) * missing_per_partial)].to_vec();

        let reduction_factor = 4;
        let scratch = vec![Fr::default(); n * 3];
        while partial_count > 1 {
            let reduced_count = (partial_count + reduction_factor - 1) / reduction_factor;
            let partial_size = next_power_of_two(partials[0].coeffs.len());
            for i in 0..reduced_count {
                let start = i * reduction_factor;
                let out_end = min((start + reduction_factor) * partial_size, n);
                let reduced_len = min(out_end - start * partial_size, zero_eval.len());
                let partials_num = min(reduction_factor, partial_count - start);
                partials[i].coeffs = work[(start * partial_size)..(start * partial_size + partials[i].coeffs.len())].to_vec();
                if partials_num > 1 {
                    let mut ret: Poly = Poly { coeffs: vec![Fr::default(); reduced_len] };
                    reduce_partials(&mut ret, &scratch, &partials[start..(start + partials_num)], fft_settings)?;
                    ret.coeffs.append(&mut partials[i].coeffs[reduced_len..partials[i].coeffs.len()].to_vec());
                    partials[i].coeffs = ret.coeffs;
                } else {
                    partials[i].coeffs = partials[i].coeffs[..partials[start].coeffs.len()].to_vec();
                }
            }

            partial_count = reduced_count;
        }

        pad_poly(&mut zero_poly.coeffs, &partials[0])?;
        fft_fr(zero_eval, &zero_poly.coeffs, false, fft_settings)?;
        zero_poly.coeffs = zero_poly.coeffs[..partials[0].coeffs.len()].to_vec();
    }

    return Ok(());
}
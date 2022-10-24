//use crate::data_types::fr::Fr;
//use crate::data_types::g1::G1;
//use kzg::{KZGSettings, Fr, Poly, G1};
use crate::data_types::g1::mclBnG1_mulVec;
use crate::data_types::{fr::*, g1::*, g2::*};
use crate::kzg10::{Curve, Polynomial};
use crate::kzg_settings::KZGSettings;
use std::convert::TryInto;

// [x] bytes_to_bls_field
// [x] vector_lincomb
// [x] g1_lincomb
// [x] blob_to_kzg_commitment
// [x] verify_kzg_proof
// [ ] compute_kzg_proof
// [x] evaluate_polynomial_in_evaluation_form

pub fn bytes_to_bls_field(bytes: &[u8; 32usize]) -> Fr {
    return Fr::from_scalar(bytes)
}

pub fn vector_lincomb(out: &mut [Fr], vectors: &[Fr], scalars: &[Fr], n: usize, m: usize) {
    let mut tmp: Fr = Fr::default();
    for j in 0..m {
        out[j] = Fr::zero();
    }
    for i in 0..n {
        for j in 0..m {
            Fr::mul(&mut tmp, &scalars[i], &vectors[i * m + j]);
            let t: Fr = out[j];
            Fr::add(&mut out[j], &t, &tmp);
        }
    }
}

pub fn g1_lincomb(out: &mut G1, p: &[G1], coeffs: &[Fr], len: usize) {
    return g1_linear_combination(out, p, coeffs, len);
}

pub fn blob_to_kzg_commitment(out: &mut G1, blob: Vec<Fr>, s: &KZGSettings) {
    g1_lincomb(out, &s.curve.g1_points, &blob, s.curve.g1_points.len());
}

pub fn verify_kzg_proof(
    out: &mut bool,
    commitment: &G1,
    x: &Fr,
    y: &Fr,
    proof: &G1,
    ks: &KZGSettings,
) {
    let (mut x_g2, mut s_minus_x) = (G2::default(), G2::default());
    let (mut y_g1, mut commitment_minus_y) = (G1::default(), G1::default());

    G2::mul(&mut x_g2, &G2::gen(), x);
    G2::sub(&mut s_minus_x, &ks.curve.g2_points[1], &x_g2);
    G1::mul(&mut y_g1, &G1::gen(), y);
    G1::sub(&mut commitment_minus_y, commitment, &y_g1);

    *out = Curve::verify_pairing(&commitment_minus_y, &G2::gen(), proof, &s_minus_x);
}

// TODO: add return value
pub fn evaluate_polynomial_in_evaluation_form(
    out: &mut Fr,
    p: &Polynomial,
    x: &Fr,
    s: &KZGSettings,
) {
    let mut tmp = Fr::default();
    let mut t: Fr;
    let plen = p.coeffs.len();
    let mut inverses_in = vec![Fr::default(); plen];
    let mut inverses = vec![Fr::default(); plen];
    let roots_of_unity = &s.fft_settings.exp_roots_of_unity;

    for i in 0..plen {
        if *x == roots_of_unity[i] {
            *out = p.coeffs[i]
        }
        Fr::sub(&mut inverses_in[i], x, &roots_of_unity[i]);
    }

    fr_batch_inv(inverses.as_mut_slice(), inverses_in.as_slice(), plen);

    *out = Fr::zero();

    for i in 0..plen {
        Fr::mul(&mut tmp, &inverses[i], &roots_of_unity[i]);
        t = tmp;
        Fr::mul(&mut tmp, &t, &p.coeffs[i]);
        t = *out;
        Fr::add(out, &t, &tmp);
    }

    let arr: [u64; 4] = [plen.try_into().unwrap(), 0, 0, 0];
    tmp = Fr::from_u64_arr(&arr);
    t = *out;
    Fr::div(out, &t, &tmp);
    tmp = x.pow(plen);
    t = tmp;
    Fr::sub(&mut tmp, &t, &Fr::one());
    t = *out;
    Fr::mul(out, &t, &tmp);
}

// TODO: add return value
fn fr_batch_inv(out: &mut [Fr], a: &[Fr], len: usize) {
    let mut prod = vec![Fr::default(); len];

    prod[0] = a[0];

    for i in 1..len {
        let t = prod[i - 1];
        Fr::mul(&mut prod[i], &a[i], &t);
    }

    let mut inv = prod[len - 1].inverse();

    let t = inv;
    for i in (1..len).rev() {
        Fr::mul(&mut out[i], &inv, &prod[i - 1]);
        Fr::mul(&mut inv, &a[i], &t);
    }
    out[0] = inv;
}

fn g1_linear_combination(result: &mut G1, g1_points: &[G1], coeffs: &[Fr], n: usize) {
    unsafe { mclBnG1_mulVec(result, g1_points.as_ptr(), coeffs.as_ptr(), n) }
}

use std::convert::TryInto;

use crate::fk20::reverse_bit_order;
use crate::poly::KzgPoly;
use crate::kzg_proofs::KZGSettings;
use crate::kzg_types::ZkG1Projective;
use crate::zkfr::blsScalar;
use kzg::{G1, Poly, Fr};

pub fn bytes_to_bls_field(bytes: &[u8; 32usize]) -> blsScalar {
    blsScalar::from_bytes(bytes).unwrap()
}

pub fn bytes_from_bls_field(fr: &blsScalar) -> [u8; 32usize] {
    fr.to_bytes()
}

pub fn compute_powers(base: &blsScalar, num_powers: usize) -> Vec<blsScalar> {
    let mut powers: Vec<blsScalar> = vec![<blsScalar as Fr>::default(); num_powers];
    powers[0] = blsScalar::one();
    for i in 1..num_powers {
        powers[i] = powers[i - 1].mul(base);
    }
    powers
}

pub fn vector_lincomb(vectors: &[Vec<blsScalar>], scalars: &[blsScalar]) -> Vec<blsScalar> {
    let mut tmp: blsScalar;
    let mut out: Vec<blsScalar> = vec![blsScalar::zero(); vectors[0].len()];
    for (v, s) in vectors.iter().zip(scalars.iter()) {
        for (i, x) in v.iter().enumerate() {
            tmp = x.mul(s);
            out[i] = out[i].add(&tmp);
        }
    }
    out
}

pub fn g1_lincomb(points: &[ZkG1Projective], scalars: &[blsScalar]) -> ZkG1Projective {
    assert!(points.len() == scalars.len());
    let mut out = G1::default();
    g1_linear_combination(&mut out, points, scalars, points.len());
    out
}

pub fn g1_linear_combination(
    out: &mut ZkG1Projective,
    p: &[ZkG1Projective],
    coeffs: &[blsScalar],
    len: usize,
) {
    let mut tmp;
    *out = G1::generator();
    for i in 0..len {
        tmp = p[i].mul(&coeffs[i]);
        *out = out.add_or_dbl(&tmp);
    }
}

pub fn blob_to_kzg_commitment(blob: &[blsScalar], s: &KZGSettings) -> ZkG1Projective {
    g1_lincomb(&s.secret_g1, blob)
}

pub fn fr_batch_inv(out: &mut [blsScalar], a: &[blsScalar], len: usize) {
    let prod: &mut Vec<blsScalar> = &mut vec![<blsScalar as Fr>::default(); len];
    let mut i: usize = 1;

    prod[0] = a[0];

    while i < len {
        prod[i] = a[i].mul(&prod[i - 1]);
        i += 1;
    }

    let inv: &mut blsScalar = &mut prod[len - 1].eucl_inverse();

    i = len - 1;
    while i > 0 {
        out[i] = prod[i - 1].mul(inv);
        *inv = a[i].mul(inv);
        i -= 1;
    }
    out[0] = *inv;
}

pub fn evaluate_polynomial_in_evaluation_form(p: &KzgPoly, x: &blsScalar, s: &KZGSettings) -> blsScalar {
    let mut tmp: blsScalar;

    let mut inverses_in: Vec<blsScalar> = vec![<blsScalar as Fr>::default(); p.len()];
    let mut inverses: Vec<blsScalar> = vec![<blsScalar as Fr>::default(); p.len()];
    let mut i: usize = 0;
    let mut roots_of_unity: Vec<blsScalar> = s.fs.expanded_roots_of_unity.clone();

    reverse_bit_order(&mut roots_of_unity);

    while i < p.len() {
        if x.equals(&roots_of_unity[i]) {
            return p.get_coeff_at(i);
        }

        inverses_in[i] = x.sub(&roots_of_unity[i]);
        i += 1;
    }
    fr_batch_inv(&mut inverses, &inverses_in, p.len());

    let mut out = blsScalar::zero();
    i = 0;
    while i < p.len() {
        tmp = inverses[i].mul(&roots_of_unity[i]);
        tmp = tmp.mul(&p.coeffs[i]);
        out = out.add(&tmp);
        i += 1;
    }
    tmp = blsScalar::from_u64(p.len().try_into().unwrap());
    out = out.div(&tmp).unwrap();

    tmp = <blsScalar as Fr>::pow(x, p.len());
    tmp = tmp.sub(&blsScalar::one());
    out = out.mul(&tmp);
    out
}
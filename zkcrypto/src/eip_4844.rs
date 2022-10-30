use crate::kzg_proofs::KZGSettings;
use crate::kzg_types::ZkG1Projective;
use crate::zkfr::blsScalar;
use kzg::G1;

pub fn bytes_to_bls_field(bytes: &[u8; 32usize]) -> blsScalar {
    blsScalar::from_bytes(bytes).unwrap()
}

pub fn bytes_from_bls_field(fr: &blsScalar) -> [u8; 32usize] {
    fr.to_bytes()
}

pub fn compute_powers(base: &blsScalar, num_powers: usize) -> Vec<blsScalar> {
    let mut powers: Vec<blsScalar> = vec![blsScalar::default(); num_powers];
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

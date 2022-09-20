use crate::fftsettings::ZkFFTSettings;
use crate::poly::ZPoly as Poly;
use crate::zkfr::blsScalar as Scalar;
use core::borrow::Borrow;
use std::ops::Mul;

use crate::kzg_types::{
    pairings_verify, ZkG1Projective as G1, ZkG2Projective as G2, G1_GENERATOR, G2_GENERATOR,
};

use kzg::{FFTFr, FFTSettings, Fr, Poly as OtherPoly, G1 as _G1, G2 as _G2};

use crate::curve::multiscalar_mul::msm_variable_base;

pub struct KZGSettings {
    pub fs: ZkFFTSettings,
    pub secret_g1: Vec<G1>,
    pub secret_g2: Vec<G2>,
    pub length: u64,
}

impl Default for KZGSettings {
    fn default() -> KZGSettings {
        KZGSettings {
            fs: ZkFFTSettings::default(),
            secret_g1: Vec::new(),
            secret_g2: Vec::new(),
            length: 0,
        }
    }
}

pub fn default_kzg() -> KZGSettings {
    KZGSettings {
        fs: ZkFFTSettings::default(),
        secret_g1: Vec::new(),
        secret_g2: Vec::new(),
        length: 0,
    }
}

pub(crate) fn new_kzg_settings(
    _secret_g1: Vec<G1>,
    _secret_g2: Vec<G2>,
    secrets_len: u64,
    _fs: &ZkFFTSettings,
) -> KZGSettings {
    KZGSettings {
        fs: _fs.borrow().clone(),
        secret_g1: _secret_g1,
        secret_g2: _secret_g2,
        length: secrets_len,
    }
}

pub fn generate_trusted_setup(n: usize, secret: [u8; 32usize]) -> (Vec<G1>, Vec<G2>) {
    let s = Scalar::from_bytes(&secret).unwrap();
    let mut s_pow = Scalar::one();

    let mut s1 = Vec::new();
    let mut s2 = Vec::new();
    for _ in 0..n {
        s1.push(G1_GENERATOR.mul(&s_pow));
        s2.push(G2_GENERATOR.mul(&s_pow));

        s_pow = s_pow.mul(&s);
    }

    (s1, s2)
}

pub(crate) fn commit_to_poly(p: &Poly, ks: &KZGSettings) -> Result<G1, String> {
    if p.coeffs.len() > ks.secret_g1.len() {
        Err(String::from("Poly given is too long"))
    } else if p.is_zero() {
        Ok(G1::identity())
    } else {
        Ok(msm_variable_base(&ks.secret_g1, &p.coeffs))
    }
}

pub(crate) fn compute_proof_single(p: &Poly, x: &Scalar, ks: &KZGSettings) -> Result<G1, String> {
    compute_proof_multi(p, x, 1, ks)
}

pub(crate) fn check_proof_single(
    com: &G1,
    proof: &G1,
    x: &Scalar,
    value: &Scalar,
    ks: &KZGSettings,
) -> Result<bool, String> {
    let x_g2: G2 = G2_GENERATOR.mul(x);
    let s_minus_x: G2 = ks.secret_g2[1].sub(&x_g2);
    let y_g1 = G1_GENERATOR.mul(value);
    let commitment_minus_y: G1 = com.sub(&y_g1);

    Ok(pairings_verify(
        &commitment_minus_y,
        &G2_GENERATOR,
        proof,
        &s_minus_x,
    ))
}

fn is_power_of_two(x: usize) -> bool {
    (x != 0) && ((x & (x - 1)) == 0)
}

pub(crate) fn compute_proof_multi(
    p: &Poly,
    x: &Scalar,
    n: usize,
    ks: &KZGSettings,
) -> Result<G1, String> {
    if !is_power_of_two(n) {
        return Err(String::from("n has to be power of two"));
    }

    let mut p2: Poly = Poly { coeffs: Vec::new() };

    let x_power_n = <Scalar as Fr>::pow(x, n);
    p2.coeffs.push(x_power_n.negate());

    for _ in 1..n {
        p2.coeffs.push(Scalar::zero());
    }

    p2.coeffs.push(Scalar::one());

    let mut p = p.clone();
    let q = p.div(&p2).unwrap();
    Ok(commit_to_poly(&q, ks).unwrap())
}

pub(crate) fn check_proof_multi(
    com: &G1,
    proof: &G1,
    x: &Scalar,
    values: &[Scalar],
    n: usize,
    ks: &KZGSettings,
) -> Result<bool, String> {
    if !is_power_of_two(n) {
        return Err(String::from("n has to be power of two"));
    }

    let mut poly = Poly {
        coeffs: ks.fs.fft_fr(values, true)?,
    };

    let x_inverse = x.inverse();
    let mut x_inverse_power = x_inverse;
    for i in 1..n {
        poly.coeffs[i] = poly.coeffs[i].mul(&x_inverse_power);
        x_inverse_power = x_inverse_power.mul(&x_inverse);
    }

    let x_power = x_inverse_power.inverse();
    let xn2 = G2_GENERATOR.mul(&x_power);
    let xn_minus_yn = ks.secret_g2[n].sub(&xn2);

    let g1 = commit_to_poly(&poly, ks).unwrap();
    let commit_minus_interp = com.sub(&g1);

    Ok(pairings_verify(
        &commit_minus_interp,
        &G2_GENERATOR,
        proof,
        &xn_minus_yn,
    ))
}

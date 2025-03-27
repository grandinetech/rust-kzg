#![allow(non_camel_case_types)]
use crate::kzg_types::{ZFp, ZFr, ZG1Affine};
use crate::kzg_types::{ZFr as BlstFr, ZG1, ZG2};
use crate::poly::PolyData;
use bls12_381::{
    multi_miller_loop, Fp12 as ZFp12, G1Affine, G2Affine, G2Prepared, MillerLoopResult,
};
use kzg::common_utils::log2_pow2;
use kzg::eip_4844::hash_to_bls_field;
use kzg::msm::precompute::PrecomputationTable;
use kzg::{FFTSettings as _, Fr as FrTrait, G1Mul, G2Mul, FFTG1, G1, G2};
use std::ops::{Add, Neg};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct FFTSettings {
    pub max_width: usize,
    pub root_of_unity: ZFr,
    pub roots_of_unity: Vec<ZFr>,
    pub brp_roots_of_unity: Vec<ZFr>,
    pub reverse_roots_of_unity: Vec<ZFr>,
}

pub fn expand_root_of_unity(root: &BlstFr, width: usize) -> Result<Vec<BlstFr>, String> {
    let mut generated_powers = vec![BlstFr::one(), *root];

    while !(generated_powers.last().unwrap().is_one()) {
        if generated_powers.len() > width {
            return Err(String::from("Root of unity multiplied for too long"));
        }

        generated_powers.push(generated_powers.last().unwrap().mul(root));
    }

    if generated_powers.len() != width + 1 {
        return Err(String::from("Root of unity has invalid scale"));
    }

    Ok(generated_powers)
}

#[derive(Debug, Clone, Default)]
pub struct KZGSettings {
    pub fs: FFTSettings,
    pub g1_values_monomial: Vec<ZG1>,
    pub g1_values_lagrange_brp: Vec<ZG1>,
    pub g2_values_monomial: Vec<ZG2>,
    pub precomputation: Option<Arc<PrecomputationTable<ZFr, ZG1, ZFp, ZG1Affine>>>,
    pub x_ext_fft_columns: Vec<Vec<ZG1>>,
    pub cell_size: usize,
}

pub fn generate_trusted_setup(len: usize, secret: [u8; 32usize]) -> (Vec<ZG1>, Vec<ZG1>, Vec<ZG2>) {
    let s = hash_to_bls_field(&secret);
    let mut s_pow = ZFr::one();

    let mut g1_monomial_values = Vec::with_capacity(len);
    let mut g2_monomial_values = Vec::with_capacity(len);

    for _ in 0..len {
        g1_monomial_values.push(ZG1::generator().mul(&s_pow));
        g2_monomial_values.push(ZG2::generator().mul(&s_pow));

        s_pow = s_pow.mul(&s);
    }

    let s = FFTSettings::new(log2_pow2(len)).unwrap();
    let g1_lagrange_values = s.fft_g1(&g1_monomial_values, true).unwrap();

    (g1_monomial_values, g1_lagrange_values, g2_monomial_values)
}

pub fn eval_poly(p: &PolyData, x: &ZFr) -> ZFr {
    if p.coeffs.is_empty() {
        return ZFr::zero();
    } else if x.is_zero() {
        return p.coeffs[0];
    }

    let mut out = p.coeffs[p.coeffs.len() - 1];
    let mut i = p.coeffs.len() - 2;

    loop {
        let temp = out.mul(x);
        out = temp.add(&p.coeffs[i]);

        if i == 0 {
            break;
        }
        i -= 1;
    }
    out
}

pub fn pairings_verify(a1: &ZG1, a2: &ZG2, b1: &ZG1, b2: &ZG2) -> bool {
    let a1neg = a1.proj.neg();

    let aa1 = G1Affine::from(&a1neg);
    let bb1 = G1Affine::from(b1.proj);
    let aa2 = G2Affine::from(a2.proj);
    let bb2 = G2Affine::from(b2.proj);

    let aa2_prepared = G2Prepared::from(aa2);
    let bb2_prepared = G2Prepared::from(bb2);

    let loop0 = multi_miller_loop(&[(&aa1, &aa2_prepared)]);
    let loop1 = multi_miller_loop(&[(&bb1, &bb2_prepared)]);

    let gt_point = loop0.add(loop1);

    let new_point = MillerLoopResult::final_exponentiation(&gt_point);

    ZFp12::eq(&ZFp12::one(), &new_point.0)
}

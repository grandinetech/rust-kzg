#![allow(non_camel_case_types)]

extern crate alloc;
use super::utils::PolyData;
use crate::kzg_types::{ArkFp, ArkFr, ArkG1Affine};
use crate::kzg_types::{ArkFr as BlstFr, ArkG1, ArkG2};
use alloc::sync::Arc;
use ark_bls12_381::Bls12_381;
use ark_ec::{PairingEngine, ProjectiveCurve};
use ark_poly::Polynomial;
use ark_std::{vec, One};
use blst::{
    blst_fp12_is_one, blst_p1_affine, blst_p1_cneg, blst_p1_to_affine, blst_p2_affine,
    blst_p2_to_affine,
};
use kzg::eip_4844::hash_to_bls_field;
use kzg::msm::precompute::PrecomputationTable;
use kzg::{Fr, G1Mul, G2Mul, G1, G2};
use std::ops::Neg;

#[derive(Debug, Clone)]
pub struct LFFTSettings {
    pub max_width: usize,
    pub root_of_unity: BlstFr,
    pub brp_roots_of_unity: Vec<BlstFr>,
    pub reverse_roots_of_unity: Vec<BlstFr>,
    pub roots_of_unity: Vec<BlstFr>,
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
pub struct LKZGSettings {
    pub fs: LFFTSettings,
    pub g1_values_monomial: Vec<ArkG1>,
    pub g1_values_lagrange_brp: Vec<ArkG1>,
    pub g2_values_monomial: Vec<ArkG2>,
    pub precomputation: Option<Arc<PrecomputationTable<ArkFr, ArkG1, ArkFp, ArkG1Affine>>>,
    pub x_ext_fft_columns: Vec<Vec<ArkG1>>,
}

pub fn generate_trusted_setup(
    n: usize,
    secret: [u8; 32usize],
) -> (Vec<ArkG1>, Vec<ArkG1>, Vec<ArkG2>) {
    let s = hash_to_bls_field(&secret);
    let mut s_pow = Fr::one();

    let mut s1 = Vec::with_capacity(n);
    let mut s2 = Vec::with_capacity(n);
    let mut s3 = Vec::with_capacity(n);

    for _ in 0..n {
        s1.push(ArkG1::generator().mul(&s_pow));
        s2.push(ArkG1::generator()); // TODO: this should be lagrange form
        s3.push(ArkG2::generator().mul(&s_pow));

        s_pow = s_pow.mul(&s);
    }

    (s1, s2, s3)
}

pub fn pairings_verify(a1: &ArkG1, a2: &ArkG2, b1: &ArkG1, b2: &ArkG2) -> bool {
    // TODO: Should look into rewriting this without blst methods. Not sure if it is possible though.
    let mut aa1 = blst_p1_affine::default();
    let mut bb1 = blst_p1_affine::default();

    let mut aa2 = blst_p2_affine::default();
    let mut bb2 = blst_p2_affine::default();

    // As an optimisation, we want to invert one of the pairings,
    // so we negate one of the points.
    let mut a1neg: ArkG1 = *a1;
    unsafe {
        blst_p1_cneg(&mut a1neg.0, true);
        blst_p1_to_affine(&mut aa1, &a1neg.0);

        blst_p1_to_affine(&mut bb1, &b1.0);
        blst_p2_to_affine(&mut aa2, &a2.0);
        blst_p2_to_affine(&mut bb2, &b2.0);

        let dst = [0u8; 3];
        let mut pairing_blst = blst::Pairing::new(false, &dst);
        pairing_blst.raw_aggregate(&aa2, &aa1);
        pairing_blst.raw_aggregate(&bb2, &bb1);
        let gt_point = pairing_blst.as_fp12().final_exp();

        blst_fp12_is_one(&gt_point)
    }
}

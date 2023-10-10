#![allow(non_camel_case_types)]
use kzg::common_utils::hash_to_bls_field;
use ark_ec::pairing::Pairing;
use ark_poly_commit::*;
use ark_std::{
    vec, One,
};
use crate::kzg_types::ArkFr;
use kzg::{G1Mul, G2Mul};
use super::utils::{
    blst_poly_into_pc_poly,
    PolyData,
};
use crate::consts::{G1_GENERATOR, G2_GENERATOR};
use crate::kzg_types::{ArkG1, ArkG2, ArkFr as BlstFr};
use ark_bls12_381::Bls12_381;
use ark_ec::CurveGroup;
use kzg::Fr as FrTrait;
use std::ops::Neg;

#[derive(Debug, Clone)]
pub struct FFTSettings {
    pub max_width: usize,
    pub root_of_unity: BlstFr,
    pub expanded_roots_of_unity: Vec<BlstFr>,
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

#[derive(Debug, Clone)]
pub struct KZGSettings {
    pub fs: FFTSettings,
    pub secret_g1: Vec<ArkG1>,
    pub secret_g2: Vec<ArkG2>,
}

impl Default for KZGSettings {
    fn default() -> Self {
        Self {
            fs: FFTSettings::default(),
            secret_g1: Vec::new(),
            secret_g2: Vec::new(),
        }
    }
}

pub fn generate_trusted_setup(len: usize, secret: [u8; 32usize]) -> (Vec<ArkG1>, Vec<ArkG2>) {
    let s = hash_to_bls_field::<ArkFr>(&secret);
    let mut s_pow = ArkFr::one();

    let mut s1 = Vec::with_capacity(len);
    let mut s2 = Vec::with_capacity(len);

    for _ in 0..len {
        s1.push(G1_GENERATOR.mul(&s_pow));
        s2.push(G2_GENERATOR.mul(&s_pow));

        s_pow = s_pow.mul(&s);
    }

    (s1, s2)
}

pub fn eval_poly(p: &PolyData, x: &BlstFr) -> BlstFr {
    let poly = blst_poly_into_pc_poly(&p.coeffs);
    BlstFr { fr: poly.evaluate(&x.fr) }
}

pub fn pairings_verify(a1: &ArkG1, a2: &ArkG2, b1: &ArkG1, b2: &ArkG2) -> bool {
    let ark_a1_neg = a1.proj
        .neg()
        .into_affine();
    let ark_b1 = b1.proj.into_affine();
    let ark_a2 = a2.proj.into_affine();
    let ark_b2 = b2.proj.into_affine();
        
    Bls12_381::multi_pairing([ark_a1_neg, ark_b1], [ark_a2, ark_b2]).0.is_one()
}

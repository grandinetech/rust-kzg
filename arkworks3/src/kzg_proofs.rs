#![allow(non_camel_case_types)]

extern crate alloc;
use crate::kzg_types::{ArkFp, ArkFr, ArkG1Affine};
use crate::kzg_types::{ArkFr as BlstFr, ArkG1, ArkG2};
use alloc::sync::Arc;
use ark_bls12_381::Bls12_381;
use ark_ec::PairingEngine;
use ark_ec::ProjectiveCurve;
use ark_std::One;
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
    let ark_a1_neg = a1.0.neg().into_affine();
    let ark_b1 = b1.0.into_affine();
    let ark_a2 = a2.0.into_affine();
    let ark_b2 = b2.0.into_affine();

    Bls12_381::product_of_pairings(&[
        (ark_a1_neg.into(), ark_a2.into()),
        (ark_b1.into(), ark_b2.into()),
    ])
    .is_one()
}

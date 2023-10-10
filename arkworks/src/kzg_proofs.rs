#![allow(non_camel_case_types)]

use ark_serialize::Valid;
use rand::SeedableRng;
use std::borrow::Borrow;

use ark_ec::VariableBaseMSM;
use ark_ec::pairing::Pairing;
use ark_ec::scalar_mul::fixed_base::FixedBase;
use ark_poly::GeneralEvaluationDomain;
use ark_poly_commit::*;
use ark_std::{
    marker::PhantomData, ops::Div, rand::RngCore, test_rng, vec, One, UniformRand, Zero,
};

use kzg::cfg_into_iter;

use super::utils::{
    blst_fr_into_pc_fr, blst_p1_into_pc_g1projective,
    blst_poly_into_pc_poly,
    PolyData,
};
use crate::consts::{G1_GENERATOR, G1_IDENTITY, G2_GENERATOR};
use crate::kzg_types::{ArkG1, ArkG2, ArkFr as BlstFr};
use crate::utils::blst_p2_into_pc_g2projective;
use ark_bls12_381::{g1, g2, Bls12_381, Fr};
use ark_ec::CurveGroup;
use ark_ec::{
    models::short_weierstrass::Affine,
    models::short_weierstrass::Projective,
    // ProjectiveCurve,
};
use ark_ff::{BigInteger256, PrimeField};
use ark_poly::univariate::DensePolynomial as DensePoly;
use kzg::{FFTFr, Fr as FrTrait, Poly};
use rand::rngs::StdRng;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::ops::{MulAssign, Neg};

use ark_ec::AffineRepr;
use std::ops::Mul;

// pub type UniPoly_381 = DensePoly<<Bls12_381 as Pairing>::ScalarField>;
// pub struct setup_type {
//     // pub params: UniversalParams<Bls12_381>,
//     pub g1_secret: Vec<Projective<g1::Config>>,
//     pub g2_secret: Vec<Projective<g2::Config>>,
// }

#[derive(Debug, Clone)]
pub struct FFTSettings {
    pub max_width: usize,
    pub root_of_unity: BlstFr,
    pub expanded_roots_of_unity: Vec<BlstFr>,
    pub reverse_roots_of_unity: Vec<BlstFr>,
    pub roots_of_unity: Vec<BlstFr>,
    // pub domain: GeneralEvaluationDomain<Fr>,
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

fn read_be_u64(input: &mut &[u8]) -> u64 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u64>());
    *input = rest;
    u64::from_be_bytes(int_bytes.try_into().unwrap())
}

pub fn generate_trusted_setup(len: usize, secret: [u8; 32usize]) -> (Vec<ArkG1>, Vec<ArkG2>) {
    let mut s_pow = Fr::from(1);
    let mut temp = vec![0; 4];
    for i in 0..4 {
        temp[i] = read_be_u64(&mut &secret[i * 8..(i + 1) * 8]);
    }
    let s = Fr::new_unchecked(BigInteger256::new([temp[0], temp[1], temp[2], temp[3]]));
    let mut s1 = Vec::new();
    let mut s2 = Vec::new();
    for _i in 0..len {
        let mut temp =
            g1::G1Affine::new_unchecked(g1::G1_GENERATOR_X, g1::G1_GENERATOR_Y).into_group();
        temp.mul_assign(s_pow);
        s1.push(ArkG1{proj: temp});
        let mut temp =
            g2::G2Affine::new_unchecked(g2::G2_GENERATOR_X, g2::G2_GENERATOR_Y).into_group();
        temp.mul_assign(s_pow);
        s2.push(ArkG2{proj: temp});
        s_pow *= s;
    }
    (s1, s2)
}

fn generate_trusted_setup_test(
    len: usize,
    s: Fr,
) -> (
    Vec<Projective<g1::Config>>,
    Vec<Projective<g2::Config>>,
) {
    let mut s_pow = Fr::from(1);
    let mut s1 = Vec::new();
    let mut s2 = Vec::new();
    for _i in 0..len {
        let mut temp = G1_GENERATOR.proj;
        temp.mul_assign(s_pow);
        s1.push(temp);
        let mut temp = G2_GENERATOR.proj;
        temp.mul_assign(s_pow);
        s2.push(temp);
        s_pow *= s;
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

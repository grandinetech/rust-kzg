#![allow(non_camel_case_types)]

extern crate alloc;
use crate::kzg_types::{ArkFp, ArkFr, ArkG1, ArkG1Affine, ArkG2};
use crate::utils::{blst_poly_into_pc_poly, fft_settings_to_rust, PolyData, PRECOMPUTATION_TABLES};
use alloc::sync::Arc;
use ark_bls12_381::Bls12_381;
use ark_ec::PairingEngine;
use ark_ec::ProjectiveCurve;
use ark_poly::Polynomial;
use ark_std::One;
use kzg::eip_4844::hash_to_bls_field;
use kzg::eth::c_bindings::CKZGSettings;
use kzg::msm::precompute::PrecomputationTable;
use kzg::{eth, Fr, G1Mul, G2Mul, G1, G2};
use std::ops::Neg;

#[derive(Debug, Clone)]
pub struct FFTSettings {
    pub max_width: usize,
    pub root_of_unity: ArkFr,
    pub brp_roots_of_unity: Vec<ArkFr>,
    pub reverse_roots_of_unity: Vec<ArkFr>,
    pub roots_of_unity: Vec<ArkFr>,
}

pub fn expand_root_of_unity(root: &ArkFr, width: usize) -> Result<Vec<ArkFr>, String> {
    let mut generated_powers = vec![ArkFr::one(), *root];

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
    pub g1_values_monomial: Vec<ArkG1>,
    pub g1_values_lagrange_brp: Vec<ArkG1>,
    pub g2_values_monomial: Vec<ArkG2>,
    pub precomputation: Option<Arc<PrecomputationTable<ArkFr, ArkG1, ArkFp, ArkG1Affine>>>,
    pub x_ext_fft_columns: Vec<Vec<ArkG1>>,
    pub cell_size: usize,
}

impl<'a> TryFrom<&'a CKZGSettings> for KZGSettings {
    type Error = String;

    fn try_from(c_settings: &CKZGSettings) -> Result<KZGSettings, String> {
        Ok(KZGSettings {
            fs: fft_settings_to_rust(c_settings)?,
            g1_values_monomial: unsafe {
                core::slice::from_raw_parts(
                    c_settings.g1_values_monomial,
                    eth::FIELD_ELEMENTS_PER_BLOB,
                )
            }
            .iter()
            .map(|r| ArkG1::from_blst_p1(*r))
            .collect::<Vec<_>>(),
            g1_values_lagrange_brp: unsafe {
                core::slice::from_raw_parts(
                    c_settings.g1_values_lagrange_brp,
                    eth::FIELD_ELEMENTS_PER_BLOB,
                )
            }
            .iter()
            .map(|r| ArkG1::from_blst_p1(*r))
            .collect::<Vec<_>>(),
            g2_values_monomial: unsafe {
                core::slice::from_raw_parts(
                    c_settings.g2_values_monomial,
                    eth::TRUSTED_SETUP_NUM_G2_POINTS,
                )
            }
            .iter()
            .map(|r| ArkG2::from_blst_p2(*r))
            .collect::<Vec<_>>(),
            x_ext_fft_columns: unsafe {
                core::slice::from_raw_parts(
                    c_settings.x_ext_fft_columns,
                    2 * ((eth::FIELD_ELEMENTS_PER_EXT_BLOB / 2) / eth::FIELD_ELEMENTS_PER_CELL),
                )
            }
            .iter()
            .map(|it| {
                unsafe { core::slice::from_raw_parts(*it, eth::FIELD_ELEMENTS_PER_CELL) }
                    .iter()
                    .map(|it| ArkG1::from_blst_p1(*it))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
            precomputation: unsafe { PRECOMPUTATION_TABLES.get_precomputation(c_settings) },
            cell_size: eth::FIELD_ELEMENTS_PER_CELL,
        })
    }
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
        s2.push(ArkG1::generator().mul(&s_pow)); // TODO: this should be lagrange form
        s3.push(ArkG2::generator().mul(&s_pow));

        s_pow = s_pow.mul(&s);
    }

    (s1, s2, s3)
}

pub fn eval_poly(p: &PolyData, x: &ArkFr) -> ArkFr {
    let poly = blst_poly_into_pc_poly(&p.coeffs);
    ArkFr {
        fr: poly.evaluate(&x.fr),
    }
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

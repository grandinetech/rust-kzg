extern crate alloc;

use kzg::{
    eip_4844::PrecomputationTableManager,
    eth::{
        self,
        c_bindings::{blst_fp, blst_fp2, blst_fr, blst_p1, blst_p2, CKZGSettings},
    },
};

use crate::kzg_types::{ArkFp, ArkFr, ArkG1, ArkG1Affine};
use crate::{kzg_proofs::FFTSettings as LFFTSettings, kzg_types::ArkG1ProjAddAffine};

use ark_bls12_381::{g1, g2, Fq, Fq2};
use ark_ec::models::short_weierstrass_jacobian::GroupProjective;
use ark_ff::{BigInteger256, BigInteger384, Fp2};
use ark_poly::univariate::DensePolynomial as DensePoly;
use ark_poly::UVPolynomial;

#[derive(Debug, PartialEq, Eq)]
pub struct Error;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct PolyData {
    pub coeffs: Vec<ArkFr>,
}
// FIXME: Store just dense poly here

pub(crate) fn fft_settings_to_rust(
    c_settings: *const CKZGSettings,
) -> Result<LFFTSettings, String> {
    let settings = unsafe { &*c_settings };

    let roots_of_unity = unsafe {
        core::slice::from_raw_parts(
            settings.roots_of_unity,
            eth::FIELD_ELEMENTS_PER_EXT_BLOB + 1,
        )
        .iter()
        .map(|r| ArkFr::from_blst_fr(*r))
        .collect::<Vec<ArkFr>>()
    };

    let brp_roots_of_unity = unsafe {
        core::slice::from_raw_parts(
            settings.brp_roots_of_unity,
            eth::FIELD_ELEMENTS_PER_EXT_BLOB,
        )
        .iter()
        .map(|r| ArkFr::from_blst_fr(*r))
        .collect::<Vec<ArkFr>>()
    };

    let reverse_roots_of_unity = unsafe {
        core::slice::from_raw_parts(
            settings.reverse_roots_of_unity,
            eth::FIELD_ELEMENTS_PER_EXT_BLOB + 1,
        )
        .iter()
        .map(|r| ArkFr::from_blst_fr(*r))
        .collect::<Vec<ArkFr>>()
    };

    Ok(LFFTSettings {
        max_width: eth::FIELD_ELEMENTS_PER_EXT_BLOB,
        root_of_unity: roots_of_unity[1],
        roots_of_unity,
        brp_roots_of_unity,
        reverse_roots_of_unity,
    })
}

pub(crate) static mut PRECOMPUTATION_TABLES: PrecomputationTableManager<
    ArkFr,
    ArkG1,
    ArkFp,
    ArkG1Affine,
    ArkG1ProjAddAffine,
> = PrecomputationTableManager::new();

pub fn pc_poly_into_blst_poly(poly: DensePoly<ark_bls12_381::Fr>) -> PolyData {
    let mut bls_pol: Vec<ArkFr> = { Vec::new() };
    for x in poly.coeffs {
        bls_pol.push(ArkFr { fr: x });
    }
    PolyData { coeffs: bls_pol }
}

pub fn blst_poly_into_pc_poly(pd: &[ArkFr]) -> DensePoly<ark_bls12_381::Fr> {
    let mut poly: Vec<ark_bls12_381::Fr> = vec![ark_bls12_381::Fr::default(); pd.len()];
    for i in 0..pd.len() {
        poly[i] = pd[i].fr;
    }
    DensePoly::from_coefficients_vec(poly)
}

pub const fn pc_fq_into_blst_fp(fq: Fq) -> blst_fp {
    blst_fp { l: fq.0 .0 }
}

pub const fn blst_fr_into_pc_fr(fr: blst_fr) -> ark_bls12_381::Fr {
    let big_int = BigInteger256::new(fr.l);

    ark_bls12_381::Fr::new(big_int)
}

pub const fn pc_fr_into_blst_fr(fr: ark_bls12_381::Fr) -> blst_fr {
    blst_fr { l: fr.0 .0 }
}

pub const fn blst_fp_into_pc_fq(fp: &blst_fp) -> Fq {
    let big_int = BigInteger384::new(fp.l);
    Fq::new(big_int)
}

pub fn blst_fp2_into_pc_fq2(fp: &blst_fp2) -> Fq2 {
    Fp2::new(blst_fp_into_pc_fq(&fp.fp[0]), blst_fp_into_pc_fq(&fp.fp[1]))
}

pub fn blst_p1_into_pc_g1projective(p1: &blst_p1) -> GroupProjective<g1::Parameters> {
    GroupProjective::new(
        blst_fp_into_pc_fq(&p1.x),
        blst_fp_into_pc_fq(&p1.y),
        blst_fp_into_pc_fq(&p1.z),
    )
}

pub const fn pc_g1projective_into_blst_p1(p1: GroupProjective<g1::Parameters>) -> blst_p1 {
    blst_p1 {
        x: blst_fp { l: p1.x.0 .0 },
        y: blst_fp { l: p1.y.0 .0 },
        z: blst_fp { l: p1.z.0 .0 },
    }
}

pub fn blst_p2_into_pc_g2projective(p2: &blst_p2) -> GroupProjective<g2::Parameters> {
    GroupProjective::new(
        blst_fp2_into_pc_fq2(&p2.x),
        blst_fp2_into_pc_fq2(&p2.y),
        blst_fp2_into_pc_fq2(&p2.z),
    )
}

pub const fn pc_g2projective_into_blst_p2(p2: GroupProjective<g2::Parameters>) -> blst_p2 {
    blst_p2 {
        x: blst_fp2 {
            fp: [blst_fp { l: p2.x.c0.0 .0 }, blst_fp { l: p2.x.c1.0 .0 }],
        },
        y: blst_fp2 {
            fp: [blst_fp { l: p2.y.c0.0 .0 }, blst_fp { l: p2.y.c1.0 .0 }],
        },
        z: blst_fp2 {
            fp: [blst_fp { l: p2.z.c0.0 .0 }, blst_fp { l: p2.z.c1.0 .0 }],
        },
    }
}

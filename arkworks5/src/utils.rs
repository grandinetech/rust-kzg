use crate::kzg_proofs::FFTSettings;
use crate::kzg_types::{ArkFp, ArkFr, ArkG1, ArkG1Affine};
use ark_bls12_381::{g1, g2, Fq, Fq2, Fr as Bls12Fr};
use ark_ec::models::short_weierstrass::Projective;
use ark_ff::Fp2;
use ark_poly::univariate::DensePolynomial as DensePoly;
use ark_poly::DenseUVPolynomial;

use kzg::eip_4844::PrecomputationTableManager;
use kzg::eth::c_bindings::{blst_fp, blst_fp2, blst_fr, blst_p1, blst_p2, CKZGSettings};

use kzg::eth;

#[derive(Debug, PartialEq, Eq)]
pub struct Error;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct PolyData {
    pub coeffs: Vec<ArkFr>,
}
// FIXME: Store just dense poly here

pub fn pc_poly_into_blst_poly(poly: DensePoly<Bls12Fr>) -> PolyData {
    let mut bls_pol: Vec<ArkFr> = { Vec::new() };
    for x in poly.coeffs {
        bls_pol.push(ArkFr { fr: x });
    }
    PolyData { coeffs: bls_pol }
}

pub fn blst_poly_into_pc_poly(pd: &[ArkFr]) -> DensePoly<Bls12Fr> {
    let mut poly: Vec<Bls12Fr> = vec![Bls12Fr::default(); pd.len()];
    for i in 0..pd.len() {
        poly[i] = pd[i].fr;
    }
    DensePoly::from_coefficients_vec(poly)
}

pub const fn pc_fq_into_blst_fp(fq: Fq) -> blst_fp {
    blst_fp { l: fq.0 .0 }
}

pub const fn blst_fr_into_pc_fr(fr: blst_fr) -> Bls12Fr {
    Bls12Fr {
        0: ark_ff::BigInt(fr.l),
        1: core::marker::PhantomData,
    }
}

pub const fn pc_fr_into_blst_fr(fr: Bls12Fr) -> blst_fr {
    blst_fr { l: fr.0 .0 }
}

pub const fn blst_fp_into_pc_fq(fp: &blst_fp) -> Fq {
    Fq {
        0: ark_ff::BigInt(fp.l),
        1: core::marker::PhantomData,
    }
}

pub const fn blst_fp2_into_pc_fq2(fp: &blst_fp2) -> Fq2 {
    Fp2 {
        c0: blst_fp_into_pc_fq(&fp.fp[0]),
        c1: blst_fp_into_pc_fq(&fp.fp[1]),
    }
}

pub const fn blst_p1_into_pc_g1projective(p1: &blst_p1) -> Projective<g1::Config> {
    Projective {
        x: blst_fp_into_pc_fq(&p1.x),
        y: blst_fp_into_pc_fq(&p1.y),
        z: blst_fp_into_pc_fq(&p1.z),
    }
}

pub const fn pc_g1projective_into_blst_p1(p1: Projective<g1::Config>) -> blst_p1 {
    blst_p1 {
        x: blst_fp { l: p1.x.0 .0 },
        y: blst_fp { l: p1.y.0 .0 },
        z: blst_fp { l: p1.z.0 .0 },
    }
}

pub const fn blst_p2_into_pc_g2projective(p2: &blst_p2) -> Projective<g2::Config> {
    Projective {
        x: blst_fp2_into_pc_fq2(&p2.x),
        y: blst_fp2_into_pc_fq2(&p2.y),
        z: blst_fp2_into_pc_fq2(&p2.z),
    }
}

pub const fn pc_g2projective_into_blst_p2(p2: Projective<g2::Config>) -> blst_p2 {
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

pub(crate) fn fft_settings_to_rust(c_settings: *const CKZGSettings) -> Result<FFTSettings, String> {
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

    Ok(FFTSettings {
        max_width: eth::FIELD_ELEMENTS_PER_EXT_BLOB,
        root_of_unity: roots_of_unity[0],
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
> = PrecomputationTableManager::new();

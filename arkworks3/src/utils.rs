extern crate alloc;

use alloc::boxed::Box;

use kzg::{
    eip_4844::{
        Blob, CKZGSettings, PrecomputationTableManager, BYTES_PER_FIELD_ELEMENT, C_KZG_RET,
        C_KZG_RET_BADARGS, FIELD_ELEMENTS_PER_BLOB, FIELD_ELEMENTS_PER_CELL,
        FIELD_ELEMENTS_PER_EXT_BLOB, TRUSTED_SETUP_NUM_G2_POINTS,
    },
    Fr,
};

use crate::kzg_types::{ArkFp, ArkFr, ArkG1, ArkG1Affine, ArkG2, LFFTSettings, LKZGSettings};

use super::{Fp, P1};
use crate::P2;
use ark_bls12_381::{g1, g2, Fq, Fq2, Fr as _Fr};
use ark_ec::models::short_weierstrass_jacobian::GroupProjective;
use ark_ff::{BigInteger256, BigInteger384, Fp2};
use ark_poly::univariate::DensePolynomial as DensePoly;
use ark_poly::UVPolynomial;
use blst::{blst_fp, blst_fp2, blst_fr, blst_p1, blst_p2};

#[derive(Debug, PartialEq, Eq)]
pub struct Error;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct PolyData {
    pub coeffs: Vec<ArkFr>,
}
// FIXME: Store just dense poly here

pub(crate) unsafe fn deserialize_blob(blob: *const Blob) -> Result<Vec<ArkFr>, C_KZG_RET> {
    (*blob)
        .bytes
        .chunks(BYTES_PER_FIELD_ELEMENT)
        .map(|chunk| {
            let mut bytes = [0u8; BYTES_PER_FIELD_ELEMENT];
            bytes.copy_from_slice(chunk);
            if let Ok(result) = ArkFr::from_bytes(&bytes) {
                Ok(result)
            } else {
                Err(C_KZG_RET_BADARGS)
            }
        })
        .collect::<Result<Vec<ArkFr>, C_KZG_RET>>()
}

macro_rules! handle_ckzg_badargs {
    ($x: expr) => {
        match $x {
            Ok(value) => value,
            Err(_) => return kzg::eip_4844::C_KZG_RET_BADARGS,
        }
    };
}

pub(crate) use handle_ckzg_badargs;

pub(crate) fn fft_settings_to_rust(
    c_settings: *const CKZGSettings,
) -> Result<LFFTSettings, String> {
    let settings = unsafe { &*c_settings };

    let roots_of_unity = unsafe {
        core::slice::from_raw_parts(settings.roots_of_unity, FIELD_ELEMENTS_PER_EXT_BLOB + 1)
            .iter()
            .map(|r| ArkFr::from_blst_fr(*r))
            .collect::<Vec<ArkFr>>()
    };

    let brp_roots_of_unity = unsafe {
        core::slice::from_raw_parts(settings.brp_roots_of_unity, FIELD_ELEMENTS_PER_EXT_BLOB)
            .iter()
            .map(|r| ArkFr::from_blst_fr(*r))
            .collect::<Vec<ArkFr>>()
    };

    let reverse_roots_of_unity = unsafe {
        core::slice::from_raw_parts(
            settings.reverse_roots_of_unity,
            FIELD_ELEMENTS_PER_EXT_BLOB + 1,
        )
        .iter()
        .map(|r| ArkFr::from_blst_fr(*r))
        .collect::<Vec<ArkFr>>()
    };

    Ok(LFFTSettings {
        max_width: FIELD_ELEMENTS_PER_EXT_BLOB,
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
> = PrecomputationTableManager::new();

pub(crate) fn kzg_settings_to_rust(c_settings: &CKZGSettings) -> Result<LKZGSettings, String> {
    Ok(LKZGSettings {
        fs: fft_settings_to_rust(c_settings)?,
        g1_values_monomial: unsafe {
            core::slice::from_raw_parts(c_settings.g1_values_monomial, FIELD_ELEMENTS_PER_BLOB)
        }
        .iter()
        .map(|r| ArkG1::from_blst_p1(*r))
        .collect::<Vec<_>>(),
        g1_values_lagrange_brp: unsafe {
            core::slice::from_raw_parts(c_settings.g1_values_lagrange_brp, FIELD_ELEMENTS_PER_BLOB)
        }
        .iter()
        .map(|r| ArkG1::from_blst_p1(*r))
        .collect::<Vec<_>>(),
        g2_values_monomial: unsafe {
            core::slice::from_raw_parts(c_settings.g2_values_monomial, TRUSTED_SETUP_NUM_G2_POINTS)
        }
        .iter()
        .map(|r| ArkG2::from_blst_p2(*r))
        .collect::<Vec<_>>(),
        x_ext_fft_columns: unsafe {
            core::slice::from_raw_parts(
                c_settings.x_ext_fft_columns,
                2 * ((FIELD_ELEMENTS_PER_EXT_BLOB / 2) / FIELD_ELEMENTS_PER_CELL),
            )
        }
        .iter()
        .map(|it| {
            unsafe { core::slice::from_raw_parts(*it, FIELD_ELEMENTS_PER_CELL) }
                .iter()
                .map(|it| ArkG1::from_blst_p1(*it))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>(),
        precomputation: unsafe { PRECOMPUTATION_TABLES.get_precomputation(c_settings) },
    })
}

pub(crate) fn kzg_settings_to_c(rust_settings: &LKZGSettings) -> CKZGSettings {
    CKZGSettings {
        roots_of_unity: Box::leak(
            rust_settings
                .fs
                .roots_of_unity
                .iter()
                .map(|r| r.to_blst_fr())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .as_mut_ptr(),
        brp_roots_of_unity: Box::leak(
            rust_settings
                .fs
                .brp_roots_of_unity
                .iter()
                .map(|r| r.to_blst_fr())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .as_mut_ptr(),
        reverse_roots_of_unity: Box::leak(
            rust_settings
                .fs
                .reverse_roots_of_unity
                .iter()
                .map(|r| r.to_blst_fr())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .as_mut_ptr(),
        g1_values_monomial: Box::leak(
            rust_settings
                .g1_values_monomial
                .iter()
                .map(|r| r.to_blst_p1())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .as_mut_ptr(),
        g1_values_lagrange_brp: Box::leak(
            rust_settings
                .g1_values_lagrange_brp
                .iter()
                .map(|r| r.to_blst_p1())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .as_mut_ptr(),
        g2_values_monomial: Box::leak(
            rust_settings
                .g2_values_monomial
                .iter()
                .map(|r| r.to_blst_p2())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .as_mut_ptr(),
        x_ext_fft_columns: Box::leak(
            rust_settings
                .x_ext_fft_columns
                .iter()
                .map(|r| {
                    Box::leak(
                        r.iter()
                            .map(|it| it.to_blst_p1())
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                    )
                    .as_mut_ptr()
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .as_mut_ptr(),
        tables: core::ptr::null_mut(),
        wbits: 0,
        scratch_size: 0,
    }
}

pub fn pc_poly_into_blst_poly(poly: DensePoly<_Fr>) -> PolyData {
    let mut bls_pol: Vec<ArkFr> = { Vec::new() };
    for x in poly.coeffs {
        bls_pol.push(ArkFr { fr: x });
    }
    PolyData { coeffs: bls_pol }
}

pub fn blst_poly_into_pc_poly(pd: &[ArkFr]) -> DensePoly<_Fr> {
    let mut poly: Vec<_Fr> = vec![_Fr::default(); pd.len()];
    for i in 0..pd.len() {
        poly[i] = pd[i].fr;
    }
    DensePoly::from_coefficients_vec(poly)
}

pub const fn pc_fq_into_blst_fp(fq: Fq) -> Fp {
    Fp { l: fq.0 .0 }
}

pub const fn blst_fr_into_pc_fr(fr: blst_fr) -> _Fr {
    let big_int = BigInteger256::new(fr.l);

    _Fr::new(big_int)
}

pub const fn pc_fr_into_blst_fr(fr: _Fr) -> blst_fr {
    blst::blst_fr { l: fr.0 .0 }
}

pub const fn blst_fp_into_pc_fq(fp: &Fp) -> Fq {
    let big_int = BigInteger384::new(fp.l);
    Fq::new(big_int)
}

pub fn blst_fp2_into_pc_fq2(fp: &blst_fp2) -> Fq2 {
    Fp2::new(blst_fp_into_pc_fq(&fp.fp[0]), blst_fp_into_pc_fq(&fp.fp[1]))
}

pub fn blst_p1_into_pc_g1projective(p1: &P1) -> GroupProjective<g1::Parameters> {
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

pub fn blst_p2_into_pc_g2projective(p2: &P2) -> GroupProjective<g2::Parameters> {
    GroupProjective::new(
        blst_fp2_into_pc_fq2(&p2.x),
        blst_fp2_into_pc_fq2(&p2.y),
        blst_fp2_into_pc_fq2(&p2.z),
    )
}

pub const fn pc_g2projective_into_blst_p2(p2: GroupProjective<g2::Parameters>) -> blst_p2 {
    blst_p2 {
        x: blst::blst_fp2 {
            fp: [
                blst::blst_fp { l: p2.x.c0.0 .0 },
                blst::blst_fp { l: p2.x.c1.0 .0 },
            ],
        },
        y: blst::blst_fp2 {
            fp: [
                blst::blst_fp { l: p2.y.c0.0 .0 },
                blst::blst_fp { l: p2.y.c1.0 .0 },
            ],
        },
        z: blst::blst_fp2 {
            fp: [
                blst::blst_fp { l: p2.z.c0.0 .0 },
                blst::blst_fp { l: p2.z.c1.0 .0 },
            ],
        },
    }
}

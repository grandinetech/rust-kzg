use crate::{
    kzg_proofs::FFTSettings,
    kzg_types::{ZFp, ZFr, ZG1Affine, ZG1},
};
use bls12_381::{G1Projective, G2Projective, Scalar};
use kzg::{
    eip_4844::PrecomputationTableManager,
    eth::{
        self,
        c_bindings::{blst_fp, blst_fp2, blst_fr, blst_p1, blst_p2, CKZGSettings},
    },
};

#[derive(Debug, PartialEq, Eq)]
pub struct Error;

pub const fn blst_fr_into_pc_fr(fr: blst_fr) -> Scalar {
    Scalar(fr.l)
}
pub const fn pc_fr_into_blst_fr(scalar: Scalar) -> blst_fr {
    blst_fr { l: scalar.0 }
}
pub const fn blst_fp2_into_pc_fq2(fp: &blst_fp2) -> bls12_381::Fp2 {
    let c0 = bls12_381::Fp(fp.fp[0].l);
    let c1 = bls12_381::Fp(fp.fp[1].l);
    bls12_381::Fp2 { c0, c1 }
}

pub const fn blst_p1_into_pc_g1projective(p1: &blst_p1) -> G1Projective {
    let x = bls12_381::Fp(p1.x.l);
    let y = bls12_381::Fp(p1.y.l);
    let z = bls12_381::Fp(p1.z.l);
    G1Projective { x, y, z }
}

pub const fn pc_g1projective_into_blst_p1(p1: G1Projective) -> blst_p1 {
    let x = blst_fp { l: p1.x.0 };
    let y = blst_fp { l: p1.y.0 };
    let z = blst_fp { l: p1.z.0 };

    blst_p1 { x, y, z }
}

pub const fn blst_p2_into_pc_g2projective(p2: &blst_p2) -> G2Projective {
    G2Projective {
        x: blst_fp2_into_pc_fq2(&p2.x),
        y: blst_fp2_into_pc_fq2(&p2.y),
        z: blst_fp2_into_pc_fq2(&p2.z),
    }
}

pub const fn pc_g2projective_into_blst_p2(p2: G2Projective) -> blst_p2 {
    let x = blst_fp2 {
        fp: [blst_fp { l: p2.x.c0.0 }, blst_fp { l: p2.x.c1.0 }],
    };

    let y = blst_fp2 {
        fp: [blst_fp { l: p2.y.c0.0 }, blst_fp { l: p2.y.c1.0 }],
    };

    let z = blst_fp2 {
        fp: [blst_fp { l: p2.z.c0.0 }, blst_fp { l: p2.z.c1.0 }],
    };

    blst_p2 { x, y, z }
}

pub(crate) fn fft_settings_to_rust(c_settings: *const CKZGSettings) -> Result<FFTSettings, String> {
    let settings = unsafe { &*c_settings };

    let roots_of_unity = unsafe {
        core::slice::from_raw_parts(
            settings.roots_of_unity,
            eth::FIELD_ELEMENTS_PER_EXT_BLOB + 1,
        )
        .iter()
        .map(|r| ZFr::from_blst_fr(*r))
        .collect::<Vec<ZFr>>()
    };

    let brp_roots_of_unity = unsafe {
        core::slice::from_raw_parts(
            settings.brp_roots_of_unity,
            eth::FIELD_ELEMENTS_PER_EXT_BLOB,
        )
        .iter()
        .map(|r| ZFr::from_blst_fr(*r))
        .collect::<Vec<ZFr>>()
    };

    let reverse_roots_of_unity = unsafe {
        core::slice::from_raw_parts(
            settings.reverse_roots_of_unity,
            eth::FIELD_ELEMENTS_PER_EXT_BLOB + 1,
        )
        .iter()
        .map(|r| ZFr::from_blst_fr(*r))
        .collect::<Vec<ZFr>>()
    };

    Ok(FFTSettings {
        max_width: eth::FIELD_ELEMENTS_PER_EXT_BLOB,
        root_of_unity: roots_of_unity[1],
        roots_of_unity,
        brp_roots_of_unity,
        reverse_roots_of_unity,
    })
}

pub(crate) static mut PRECOMPUTATION_TABLES: PrecomputationTableManager<ZFr, ZG1, ZFp, ZG1Affine> =
    PrecomputationTableManager::new();

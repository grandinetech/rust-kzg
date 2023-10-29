use super::{P1};
use crate::P2;
use blst::{blst_fp, blst_fp2, blst_fr, blst_p1, blst_p2};
use bls12_381::{Fp as ZFp, Fp2 as ZFp2, G1Projective, G2Projective, Scalar};

#[derive(Debug, PartialEq, Eq)]
pub struct Error;

pub const fn blst_fr_into_pc_fr(fr: blst_fr) -> Scalar {
    Scalar(fr.l)
}
pub const fn pc_fr_into_blst_fr(scalar: Scalar) -> blst_fr {
    blst_fr { l: scalar.0 }
}
pub const fn blst_fp2_into_pc_fq2(fp: &blst_fp2) -> ZFp2 {
    let c0 = ZFp(fp.fp[0].l);
    let c1 = ZFp(fp.fp[1].l);
    ZFp2 { c0, c1 }
}

pub const fn blst_p1_into_pc_g1projective(p1: &P1) -> G1Projective {
    let x = ZFp(p1.x.l);
    let y = ZFp(p1.y.l);
    let z = ZFp(p1.z.l);
    G1Projective { x, y, z }
}

pub const fn pc_g1projective_into_blst_p1(p1: G1Projective) -> blst_p1 {
    let x = blst_fp { l: p1.x.0 };
    let y = blst_fp { l: p1.y.0 };
    let z = blst_fp { l: p1.z.0 };

    blst_p1 { x, y, z }
}

pub const fn blst_p2_into_pc_g2projective(p2: &P2) -> G2Projective {
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

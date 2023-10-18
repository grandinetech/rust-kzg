use super::{Fp, P1};
use crate::P2;
use crate::kzg_types::{ArkFr};
use ark_bls12_381::{g1, g2, Fq, Fr};
use ark_ec::models::short_weierstrass::Projective;
use ark_ff::{biginteger::BigInteger256, biginteger::BigInteger384, Fp2, Fp384};
use ark_poly::univariate::DensePolynomial as DensePoly;
use ark_poly::DenseUVPolynomial;
use blst::{blst_fr, blst_p1, blst_fp, blst_p2};

#[derive(Debug, PartialEq, Eq)]
pub struct Error;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct PolyData {
    pub coeffs: Vec<ArkFr>,
}
// FIXME: Store just dense poly here

pub fn pc_poly_into_blst_poly(poly: DensePoly<Fr>) -> PolyData {
    let mut bls_pol: Vec<ArkFr> = { Vec::new() };
    for x in poly.coeffs {
        bls_pol.push(ArkFr { fr: x } );
    }
    PolyData { coeffs: bls_pol }
}

pub fn blst_poly_into_pc_poly(pd: &[ArkFr]) -> DensePoly<Fr> {
    let mut poly: Vec<Fr> = vec![Fr::default(); pd.len()];
    for i in 0..pd.len() {
        poly[i] = pd[i].fr;
    }
    DensePoly::from_coefficients_vec(poly)
}

pub fn pc_fq_into_blst_fp(fq: Fq) -> Fp {
    Fp { l: fq.0 .0 }
}

pub fn blst_fr_into_pc_fr(fr: blst_fr) -> Fr {
    let big_int = BigInteger256::new(fr.l);
    Fr::new_unchecked(big_int)
}

pub fn pc_fr_into_blst_fr(fr: Fr) -> blst_fr {
    let big_int = BigInteger256::from(fr);
    blst::blst_fr { l: big_int.0 }
}

pub const fn blst_fp_into_pc_fq(fp: &Fp) -> Fq {
    let big_int = BigInteger384::new(fp.l);
    Fq::new_unchecked(big_int)
}

pub const fn blst_p1_into_pc_g1projective(p1: &P1) -> Projective<g1::Config> {
    Projective::new_unchecked(
        blst_fp_into_pc_fq(&p1.x),
        blst_fp_into_pc_fq(&p1.y),
        blst_fp_into_pc_fq(&p1.z),
    )
}

pub const fn pc_g1projective_into_blst_p1(p1: Projective<g1::Config>) -> blst_p1 {
    blst_p1 {
        x: blst_fp { l: p1.x.0.0 },
        y: blst_fp { l: p1.y.0.0 },
        z: blst_fp { l: p1.z.0.0 },
    }
}

pub const fn blst_p2_into_pc_g2projective(p2: &P2) -> Projective<g2::Config> {
    Projective::new_unchecked(
        Fp2::new(
            Fp384::new_unchecked(BigInteger384::new(p2.x.fp[0].l)),
            Fp384::new_unchecked(BigInteger384::new(p2.x.fp[1].l)),
        ),
        Fp2::new(
            Fp384::new_unchecked(BigInteger384::new(p2.y.fp[0].l)),
            Fp384::new_unchecked(BigInteger384::new(p2.y.fp[1].l)),
        ),
        Fp2::new(
            Fp384::new_unchecked(BigInteger384::new(p2.z.fp[0].l)),
            Fp384::new_unchecked(BigInteger384::new(p2.z.fp[1].l)),
        ),
    )
}

pub const fn pc_g2projective_into_blst_p2(p2: Projective<g2::Config>) -> blst_p2 {
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
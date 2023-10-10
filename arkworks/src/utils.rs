use super::{Fp, P1};
use crate::P2;
use crate::kzg_types::{ArkFr};
use ark_bls12_381::{g1, g2, Fq, Fr};
use ark_ec::models::short_weierstrass::Projective;
use ark_ff::{biginteger::BigInteger256, biginteger::BigInteger384, Fp2, Fp384};
use ark_poly::univariate::DensePolynomial as DensePoly;
use ark_poly::DenseUVPolynomial;
use blst::blst_fr;
use kzg::Poly;

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

// pub fn pc_fr_into_blst_fr(fr: Fr) -> ArkFr {
//     let temp = blst::blst_fr { l: fr.0 .0 };
//     ArkFr(temp)
// }

// pub fn pc_fq_into_blst_fp(fq: Fq) -> Fp {
//     Fp { l: fq.0 .0 }
// }

pub fn blst_fr_into_pc_fr(fr: blst_fr) -> Fr {
    let big_int = BigInteger256::new(fr.l);
    Fr::new_unchecked(big_int)
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

pub const fn blst_p2_into_pc_g2projective(p2: &P2) -> Projective<g2::Config> {
    let pc_projective = Projective::new_unchecked(
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
    );
    pc_projective
}

use super::{Fp, P1};
use crate::kzg_types::{ArkG1, ArkG2, FsFr as BlstFr};
use ark_bls12_381::{Fr, Fq, g1, g2};
use ark_ec::{
    models::short_weierstrass_jacobian::GroupProjective,
};
use ark_ff::{biginteger::BigInteger256, biginteger::BigInteger384, Fp2, Fp384};
use ark_poly::univariate::DensePolynomial as DensePoly;
use ark_poly::UVPolynomial;

#[derive(Debug, PartialEq)]
pub struct Error;

#[derive(Debug)]
pub struct PolyData {
    pub coeffs: Vec<BlstFr>,
}

pub(crate) fn pc_poly_into_blst_poly(poly: DensePoly<Fr>) -> Result<PolyData, String> {
    let mut bls_pol = PolyData { coeffs: Vec::new() };

    for x in poly.coeffs {
        bls_pol.coeffs.push(pc_fr_into_blst_fr(x));
    }

    Ok(bls_pol)
}

pub(crate) fn blst_poly_into_pc_poly(pd: &PolyData) -> Result<DensePoly<Fr>, String> {
    let mut poly = Vec::new();
    let x = pd.coeffs.clone();
    for x in x {
        poly.push(Fr::new(BigInteger256::new(x.0.l)))
    }
    let p = DensePoly::from_coefficients_slice(&poly);
    Ok(p)
}

pub(crate) fn pc_fr_into_blst_fr(fr: Fr) -> BlstFr {
    let temp = blst::blst_fr { l: fr.0 .0 };
    BlstFr(temp)
}

pub(crate) fn pc_fq_into_blst_fp(fq: Fq) -> Fp {
    Fp { l: fq.0 .0 }
}

pub(crate) fn blst_fr_into_pc_fr(fr: &BlstFr) -> Fr {
    let big_int = BigInteger256::new(fr.0.l);
    Fr::new(big_int)
}

pub(crate) fn blst_fp_into_pc_fq(fp: &Fp) -> Fq {
    let big_int = BigInteger384::new(fp.l);
    Fq::new(big_int)
}

pub(crate) fn pc_g1projective_into_blst_p1(
    gp: GroupProjective<g1::Parameters>,
) -> Result<ArkG1, Error> {
    let p1 = ArkG1(blst::blst_p1 {
        x: pc_fq_into_blst_fp(gp.x),
        y: pc_fq_into_blst_fp(gp.y),
        z: pc_fq_into_blst_fp(gp.z),
    });
    Ok(p1)
}

pub(crate) fn blst_p1_into_pc_g1projective(
    p1: &P1,
) -> Result<GroupProjective<g1::Parameters>, Error> {
    let pc_projective = GroupProjective::new(
        blst_fp_into_pc_fq(&p1.x),
        blst_fp_into_pc_fq(&p1.y),
        blst_fp_into_pc_fq(&p1.z),
    );
    Ok(pc_projective)
}

pub(crate) fn pc_g2projective_into_blst_p2(
    p2: GroupProjective<g2::Parameters>,
) -> Result<ArkG2, Error> {
    let blst_projective = ArkG2(blst::blst_p2 {
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
    });
    Ok(blst_projective)
}

pub(crate) fn blst_p2_into_pc_g2projective(
    p2: &ArkG2,
) -> Result<GroupProjective<g2::Parameters>, Error> {
    let pc_projective = GroupProjective::new(
        Fp2::new(
            Fp384::new(BigInteger384::new(p2.0.x.fp[0].l)),
            Fp384::new(BigInteger384::new(p2.0.x.fp[1].l)),
        ),
        Fp2::new(
            Fp384::new(BigInteger384::new(p2.0.y.fp[0].l)),
            Fp384::new(BigInteger384::new(p2.0.y.fp[1].l)),
        ),
        Fp2::new(
            Fp384::new(BigInteger384::new(p2.0.z.fp[0].l)),
            Fp384::new(BigInteger384::new(p2.0.z.fp[1].l)),
        ),
    );
    Ok(pc_projective)
}

// pub(crate) fn pc_affine_into_blst_affine(
//     affine: GroupAffine<g1::Parameters>,
// ) -> Result<P1Affine, Error> {
//     let bl_aff = P1Affine {
//         x: Fp { l: affine.x.0 .0 },
//         y: Fp { l: affine.y.0 .0 },
//     };

//     Ok(bl_aff)
// }

// pub(crate) fn blst_affine_into_pc_affine(
//     affine: &P1Affine,
// ) -> Result<GroupAffine<g1::Parameters>, Error> {
//     let pc_affine = GroupAffine::new(
//         blst_fp_into_pc_fq(&affine.x),
//         blst_fp_into_pc_fq(&affine.y),
//         false,
//     );
//     Ok(pc_affine)
// }

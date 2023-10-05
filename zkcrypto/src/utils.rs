//use bls12_381::{G1Affine, G1Projective, G2Affine, G2Projective};
use crate::zkfr::blsScalar; // Gal naudot crate::zkfr::blsScalar;?
                            //use bls12_381::*;
use blst::blst_fp as BlstFp;
use blst::blst_fr as BlstFr;
use blst::blst_p1 as BlstP1;
use kzg::Fr;
use std::fmt;

use core::fmt::Error;

pub use crate::curve::fp::Fp as ZkFp;
pub use crate::curve::fp12::Fp12 as ZkFp12;
pub use crate::curve::fp2::Fp2 as ZkFp2;
pub use crate::curve::g1::G1Affine as ZkG1Affine;
pub use crate::curve::g1::G1Projective as ZkG1Projective;
pub use crate::curve::g2::G2Affine as ZkG2Affine;
pub use crate::curve::g2::G2Projective as ZkG2Projective;

use crate::poly::ZPoly as Poly;

pub struct Polydata {
    coeffs: Vec<BlstFr>,
}

pub fn blst_poly_into_zk_poly(pd: Polydata) -> Result<Poly, fmt::Error> {
    //use bls12_381::Scalar as blsScalar;
    let mut poly = Vec::new();
    for x in pd.coeffs {
        poly.push(blsScalar::from(x.l[0]))
    }

    let p = Poly { coeffs: poly }; // Poly(poly)
    Ok(p)
}

pub fn pc_poly_into_blst_poly(poly: Poly) -> Result<Polydata, fmt::Error> {
    let mut bls_pol = Polydata { coeffs: Vec::new() };

    for x in poly.coeffs {
        // poly.0
        bls_pol.coeffs.push(BlstFr { l: x.0 });
    }

    Ok(bls_pol)
}

pub fn zk_fr_into_blst_fr(fr: &blsScalar) -> BlstFr {
    // BlstFr::from_u64_arr(&fr.0);
    let mut ret = blst::blst_fr::default();
    unsafe {
        blst::blst_fr_from_uint64(&mut ret, fr.0.as_ptr());
    }
    // let temp = blst::blst_fr { l: fr.0};
    // // BlstFr(temp)
    // return temp;
    blst::blst_fr { l: fr.0 }
    // BlstFr { l: fr.0 }
}

pub fn blst_fr_into_zk_fr(fr: &BlstFr) -> blsScalar {
    let mut size: [u64; 4] = [0; 4];
    unsafe {
        blst::blst_uint64_from_fr(size.as_mut_ptr(), fr);
    }
    // // blst::blst_uint64_from_fr

    // let zk_fr = blsScalar::from_u64_arr(&size);
    // zk_fr
    blsScalar::from_u64_arr(&size)
}

pub fn zk_fp_into_blst_fp(fp: ZkFp) -> BlstFp {
    BlstFp { l: fp.0 }
}

pub fn blst_fp_into_zk_fp(fp: &BlstFp) -> ZkFp {
    let mut size: [u64; 6] = [0; 6];
    unsafe {
        blst::blst_uint64_from_fp(size.as_mut_ptr(), fp);
    }
    // let zk_fp = ZkFp::from_raw_unchecked(size);
    // zk_fp
    ZkFp::from_raw_unchecked(size)
}

// turbut reikia is zkcrypto i savo repo isidet visus fp ir t.t
pub fn zk_g1projective_into_blst_p1(gp: ZkG1Projective) -> Result<BlstP1, Error> {
    let p1 = BlstP1 {
        x: zk_fp_into_blst_fp(gp.x),
        y: zk_fp_into_blst_fp(gp.y),
        z: zk_fp_into_blst_fp(gp.z),
    };
    Ok(p1)
}

pub fn blst_p1_into_zk_g1projective(p1: &BlstP1) -> Result<ZkG1Projective, Error> {
    let zk_g1projective: ZkG1Projective = ZkG1Projective {
        x: blst_fp_into_zk_fp(&p1.x),
        y: blst_fp_into_zk_fp(&p1.y),
        z: blst_fp_into_zk_fp(&p1.z),
    };
    Ok(zk_g1projective)
}

pub fn min_u64(a: usize, b: usize) -> Result<usize, String> {
    if a < b {
        Ok(a)
    } else {
        Ok(b)
    }
}

//use bls12_381::{G1Affine, G1Projective, G2Affine, G2Projective};
use crate::zkfr::blsScalar; // Gal naudot crate::zkfr::blsScalar;?
//use bls12_381::*;
use kzg::Fr;
use blst::blst_fr as BlstFr;
use blst::blst_fp as BlstFp;
use blst::blst_p1 as BlstP1;
use std::fmt;

use core::fmt::Error;

// pub use crate::kzg_types::ZkG1Affine; 
// pub use crate::kzg_types::ZkG1Projective; 
// pub use crate::kzg_types::ZkG2Affine; 
// pub use crate::kzg_types::ZkG2Projective;
// pub use crate::kzg_types::ZkFp;
// pub use crate::kzg_types::ZkFp2;
// pub use crate::kzg_types::ZkFp12;

pub use crate::curve::g1::G1Affine as ZkG1Affine; 
pub use crate::curve::g1::G1Projective as ZkG1Projective; 
pub use crate::curve::g2::G2Affine as ZkG2Affine; 
pub use crate::curve::g2::G2Projective as ZkG2Projective;
pub use crate::curve::fp::Fp as ZkFp;
pub use crate::curve::fp2::Fp2 as ZkFp2;
pub use crate::curve::fp12::Fp12 as ZkFp12;


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

    let p = Poly {coeffs: poly}; // Poly(poly)
    Ok(p)
}

pub fn pc_poly_into_blst_poly(poly: Poly) -> Result<Polydata, fmt::Error> {
        let mut bls_pol = Polydata { coeffs: Vec::new() };
		
        for x in poly.coeffs { // poly.0
            bls_pol.coeffs.push(BlstFr{l:x.0});
        }

        Ok(bls_pol)
}

pub fn zk_fr_into_blst_fr(fr: &blsScalar) -> BlstFr {
	// BlstFr::from_u64_arr(&fr.0);
	let mut ret = blst::blst_fr::default();
	unsafe {
		blst::blst_fr_from_uint64(&mut ret, fr.0.as_ptr());
	}
	let temp = blst::blst_fr { l: fr.0};
	// BlstFr(temp)
	return temp;
	// BlstFr { l: fr.0 }
}

pub fn blst_fr_into_zk_fr(fr: &BlstFr) -> blsScalar { 
	let mut size: [u64; 4] = [0; 4];
	unsafe {
		blst::blst_uint64_from_fr(size.as_mut_ptr(), fr);
	}
	// // blst::blst_uint64_from_fr
	
	let zk_fr = blsScalar::from_u64_arr(&size);
	return zk_fr;
}

pub fn zk_fp_into_blst_fp(fp: ZkFp) -> BlstFp {
    BlstFp { l: fp.0 }
}

pub fn blst_fp_into_zk_fp(fp: &BlstFp) -> ZkFp {
    let mut size: [u64; 6] = [0; 6];
	unsafe {
		blst::blst_uint64_from_fp(size.as_mut_ptr(), fp);
	}
    let zk_fp = ZkFp::from_raw_unchecked(size);
	return zk_fp;
}

// turbut reikia is zkcrypto i savo repo isidet visus fp ir t.t
pub fn zk_g1projective_into_blst_p1(
        gp: ZkG1Projective
    ) -> Result<BlstP1, Error> {
        let p1 = BlstP1{
            x: zk_fp_into_blst_fp(gp.x),
            y: zk_fp_into_blst_fp(gp.y),
            z: zk_fp_into_blst_fp(gp.z),
        };
        Ok(p1)
}

pub fn blst_p1_into_zk_g1projective(
    p1: &BlstP1,
) -> Result<ZkG1Projective, Error> {
    let zk_g1projective: ZkG1Projective = ZkG1Projective {
		x: blst_fp_into_zk_fp(&p1.x),
        y: blst_fp_into_zk_fp(&p1.y),
        z: blst_fp_into_zk_fp(&p1.z),
    };
    Ok(zk_g1projective)
}

pub fn is_power_of_two(x: usize) -> bool {
    (x != 0) && ((x & (x - 1)) == 0)
}

pub fn next_power_of_two(v: usize) -> usize {
    let mut v = v;

    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v |= v >> 32;
    v += 1;
    v += (v == 0) as usize;

    v
}

pub fn log_2_byte(b: u8) -> usize {
    let mut r = if b > 0xF { 1 } else { 0 } << 2;
    let mut b = b >> r;
    let shift = if b > 0x3 { 1 } else { 0 } << 1;
    b >>= shift + 1;
    r |= shift | b;
    r.into()
}

pub fn log2_pow2(n: usize) -> usize {
    let bytes: [usize; 5] = [0xAAAAAAAA, 0xCCCCCCCC, 0xF0F0F0F0, 0xFF00FF00, 0xFFFF0000];
    let mut r: usize = if (n & bytes[0]) != 0 {1} else {0};
    r |= if (n & bytes[1]) != 0 {1} else {0} << 1;
    r |= if (n & bytes[2]) != 0 {1} else {0} << 2;
    r |= if (n & bytes[3]) != 0 {1} else {0} << 3;
    r |= if (n & bytes[4]) != 0 {1} else {0} << 4;
    r
}

pub fn log2_u64(n: usize) -> usize {
    let mut n2 = n;
    let mut r: usize = 0;
    while (n2 >> 1) != 0 {
        n2 = n2 >> 1;
        r += 1;
    }
    r
}

pub fn min_u64(a: usize, b: usize) -> Result<usize, String> {
    return if a < b {Ok(a)} else {Ok(b)};
}
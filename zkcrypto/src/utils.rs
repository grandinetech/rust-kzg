use bls12_381::{G1Affine, G1Projective, G2Affine, G2Projective};
//use poly::blsScalar as Fr;
use crate::curve::scalar::Scalar as Fr; // Gal naudot crate::zkfr::blsScalar;?
use bls12_381::*;
// use kzg::{P1, P2, P1Affine};
use blst::blst_fr as BlstFr;
use std::fmt;
use super::*;

use crate::poly::ZPoly as Poly;
// use crate::poly::Poly;

//use pairing::Engine;

pub struct polydata {
    coeffs: Vec<BlstFr>,
}

pub fn blst_poly_into_zk_poly(pd: polydata) -> Result<Poly, fmt::Error> {
	//use bls12_381::Scalar as Fr;
    let mut poly = Vec::new();
    for x in pd.coeffs {
        poly.push(Fr::from(x.l[0]))
    }

    let p = Poly {coeffs: poly}; // Poly(poly)
    Ok(p)
}

pub(crate) fn pc_poly_into_blst_poly(poly: Poly) -> Result<polydata, fmt::Error> {
        let mut bls_pol = polydata { coeffs: Vec::new() };
		
        for x in poly.coeffs { // poly.0
            bls_pol.coeffs.push(BlstFr{l:x.0});
        }

        Ok(bls_pol)
}

pub fn zk_fr_into_blst_fr(fr: Fr) -> BlstFr {
        BlstFr { l: fr.0 }
}

pub fn blst_fr_into_zk_fr(fr: BlstFr) -> Fr { 
	Fr::from_raw(fr.l) 
}


// turbut reikia is zkcrypto i savo repo isidet visus fp ir t.t
// pub fn zk_g1projective_into_blst_p1(
        // gp: GroupProjective<g1::Parameters>
    // ) -> Result<P1, Error> {
        // let p1 = P1{
            // x: zk_fq_into_blst_fp(gp.x),
            // y: zk_fq_into_blst_fp(gp.y),
            // z: zk_fq_into_blst_fp(gp.z),
        // };
        // Ok(p1)
// }

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
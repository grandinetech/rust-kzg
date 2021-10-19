use bls12_381::{G1Affine, G1Projective, G2Affine, G2Projective};
use bls12_381::Scalar;
use bls12_381::*;
use std::fmt;

use crate::poly::Poly;

use pairing::Engine;

pub struct polydata {
    coeffs: Vec<u64>,
}


pub fn blst_poly_into_zk_poly(pd: polydata) -> Result<Poly, fmt::Error> {
	use bls12_381::Scalar as Fr;
    let mut poly = Vec::new();
    for x in pd.coeffs {
        poly.push(Fr::from(x))
    }

    let p = super::Poly(poly);
    Ok(p)
}
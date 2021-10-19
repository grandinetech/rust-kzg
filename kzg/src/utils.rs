use bls12_381::{G1Affine, G1Projective, G2Affine, G2Projective};
use bls12_381::Scalar;
use bls12_381::*;

use crate::poly::Poly;

use pairing::Engine;

type Zk_Fr = bls12_381::Scalar;

pub struct polydata {
    coeffs: Vec<u64>,
}


pub fn blst_poly_into_pc_poly(pd: polydata) -> Result<Poly, Error> {
    let mut poly = Vec::new();
    for x in pd.coeffs {
        poly.push(Fr::from(x))
    }

    let p = DensePoly::from_coefficients_slice(&poly);
    Ok(p)
}
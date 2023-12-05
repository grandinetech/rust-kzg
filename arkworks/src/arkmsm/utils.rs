use ark_bls12_381::{G1Affine, G1Projective};
use ark_ec::{AffineRepr, CurveGroup, CurveConfig};
use ark_ff::{PrimeField, UniformRand, BigInteger256};
use kzg::Fr;

use crate::kzg_types::{ArkFp, ArkFr};

#[allow(clippy::type_complexity)]
pub fn generate_msm_inputs(
    size: usize,
) -> (
    Vec<G1Affine>,
    Vec<BigInteger256>
) {
    let mut rng = ark_std::test_rng();
    let scalar_vec = (0..size)
        .map(|_| ArkFr::rand().fr.into_bigint())
        .collect();
    let point_vec = (0..size)
        .map(|_| G1Projective::rand(&mut rng))
        .collect::<Vec<_>>();
    (
        G1Projective::normalize_batch(&point_vec),
        scalar_vec,
    )
}

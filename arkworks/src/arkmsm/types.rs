use ark_bls12_381::G1Affine;
use ark_ec::{models::CurveConfig, AffineRepr};
use ark_ff::{FpConfig, PrimeField, Field};

pub const G1_SCALAR_SIZE: u32 =
    <<G1Affine as AffineRepr>::ScalarField as PrimeField>::MODULUS_BIT_SIZE;
pub const G1_SCALAR_SIZE_GLV: u32 = 128u32;
pub const GROUP_SIZE_IN_BITS: usize = 6;
pub const GROUP_SIZE: usize = 1 << GROUP_SIZE_IN_BITS;

pub type G1BigInt = <<G1Affine as AffineRepr>::ScalarField as PrimeField>::BigInt;
pub type G1Projective = <G1Affine as AffineRepr>::Group;
pub type G1ScalarField = <G1Affine as AffineRepr>::ScalarField;
pub type G1BaseField = <G1Affine as AffineRepr>::BaseField;

pub type BigInt<P> = <<P as CurveConfig>::ScalarField as PrimeField>::BigInt;

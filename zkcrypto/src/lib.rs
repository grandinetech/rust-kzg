pub type ZPoly = poly::KzgPoly;

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod consts;
pub mod das;
pub mod eip_4844;
pub mod fft_fr;
pub mod fft_g1;
pub mod fftsettings;
pub mod finite;
pub mod fk20;
pub mod kzg_proofs;
pub mod kzg_types;
pub mod poly;
pub mod recover;
pub mod utils;
pub mod zero_poly;
pub mod zkfr;
#[macro_use]
pub mod curve {
    pub mod fp;
    pub mod fp12;
    pub mod fp2;
    pub mod fp6;
    pub mod g1;
    pub mod g2;
    pub mod multiscalar_mul;
    pub mod pairings;
    pub mod scalar;
    #[cfg(feature = "pairings")]
    pub use pairings::{pairing, Bls12, Gt, MillerLoopResult};

    // #[cfg(all(feature = "pairings", feature = "alloc"))]
    pub use pairings::G2Prepared;
}
pub type BlsScalar = curve::scalar::Scalar;

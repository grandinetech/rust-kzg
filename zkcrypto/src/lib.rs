pub type ZPoly = poly::KzgPoly;

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod finite;
pub mod utils;
pub mod poly;
pub mod fftsettings;
pub mod consts;
pub mod zkfr;
pub mod fft_fr;
pub mod kzg_types;
pub mod kzg_proofs;
pub mod das;
pub mod zero_poly;
pub mod recover;
#[macro_use]
pub mod curve {
	pub mod scalar;
	pub mod g1;
    pub mod g2;
    pub mod fp;
    pub mod fp2;
    pub mod fp6;
    pub mod fp12;
	pub mod pairings;
	pub mod multiscalar_mul;

	#[cfg(feature = "pairings")]
	pub use pairings::{pairing, Bls12, Gt, MillerLoopResult};

	// #[cfg(all(feature = "pairings", feature = "alloc"))]
	pub use pairings::{ G2Prepared};
}
pub type BlsScalar = curve::scalar::Scalar;

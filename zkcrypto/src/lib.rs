pub type ZPoly = poly::KzgPoly;

pub mod finite;
pub mod utils;
pub mod poly;
pub mod fftsettings;
pub mod consts;
pub mod zkfr;
pub mod fft_fr;
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
}
pub type BlsScalar = curve::scalar::Scalar;

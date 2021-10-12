// Blst
/*pub type Pairing = blst::Pairing;
pub type Fp = blst::blst_fp;
pub type Fp12 = blst::blst_fp12;
pub type Fp6 = blst::blst_fp6;*/
pub type Fr = finite::BlstFr;
/*pub type P1 = blst::blst_p1;
pub type P1Affine = blst::blst_p1_affine;
pub type P2 = blst::blst_p2;
pub type P2Affine = blst::blst_p2_affine;
pub type Scalar = blst::blst_scalar;
pub type Uniq = blst::blst_uniq;*/
// Poly
pub type Poly = poly::KzgPoly;
pub type G1 = fftsettings::BlstP1;
pub type G2 = fftsettings::BlstP2;
pub type P1 = fftsettings::BlstP1;
pub type P2 = fftsettings::BlstP2;
pub type FFTSettings = fftsettings::FFTSettings;
// Common
pub type Error = common::KzgRet;

pub mod common;
pub mod finite;
pub mod fftsettings;
pub mod poly;
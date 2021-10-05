// Blst
/*pub type Pairing = blst::Pairing;
pub type Fp = blst::blst_fp;
pub type Fp12 = blst::blst_fp12;
pub type Fp6 = blst::blst_fp6;*/
pub type Fr = BlstFr;/*blst::blst_fr;*/
/*pub type P1 = blst::blst_p1;
pub type P1Affine = blst::blst_p1_affine;
pub type P2 = blst::blst_p2;
pub type P2Affine = blst::blst_p2_affine;
pub type Scalar = blst::blst_scalar;
pub type Uniq = blst::blst_uniq;*/
// Poly
pub type Poly = KzgPoly;
// Common
pub type Error = KzgRet;

pub mod finite;
pub mod poly;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct BlstFr {
    pub l: [u64; 4]
}

#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum KzgRet {
    KzgOk = 0,
    KzgBadArgs = 1,
    KzgError = 2,
    KzgMalloc = 3
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct KzgPoly {
    pub coeffs: *mut BlstFr,
    pub length: u64
}

impl<> Default for KzgPoly<> {
    fn default() -> Self {
        Self { coeffs: &mut BlstFr{l: [0, 0, 0, 0] }, length: 4 }
    }
}

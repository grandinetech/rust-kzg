pub type Pairing = blst::Pairing;
pub type Fp = blst::blst_fp;
pub type Fp12 = blst::blst_fp12;
pub type Fp6 = blst::blst_fp6;
pub type Fr = blst::blst_fr;
pub type P1 = blst::blst_p1;
pub type P1Affine = blst::blst_p1_affine;
pub type P2 = blst::blst_p2;
pub type P2Affine = blst::blst_p2_affine;
pub type Scalar = blst::blst_scalar;
pub type Uniq = blst::blst_uniq;

pub mod finite;

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum KzgRet {
    KzgOk = 0,
    KzgBadArgs = 1,
    KzgError = 2,
    KzgMalloc = 3
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BlstFr {
    pub l: [u64; 4]
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Poly {
    pub coeffs: *mut BlstFr,
    pub length: u64
}

#[link(name = "ckzg", kind = "static")]
#[link(name = "blst", kind = "static")]
extern "C" {
    pub fn new_poly(out: *mut Poly, length: u64) -> KzgRet;
    pub fn free_poly(p: *mut Poly);
}

#[cfg(test)]
mod tests {
    use crate::{Poly, new_poly, free_poly, KzgRet, BlstFr};

    #[test]
    fn test_poly() {
        unsafe {
            let some_l: [u64; 4] = [0, 0, 0, 0];
            let some_poly = &mut Poly{ coeffs: &mut BlstFr{l: some_l }, length: 4 };
            assert_eq!(new_poly(some_poly, 4), KzgRet::KzgOk);
            free_poly(some_poly);
        }
    }
}

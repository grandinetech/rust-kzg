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

pub mod finite;
pub mod fftsettings;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct BlstFr {
    pub l: [u64; 4]
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FFTSettings {
    pub max_width: u64,
    pub root_of_unity: *mut BlstFr,
    pub expanded_roots_of_unity: *mut BlstFr,
    pub reverse_roots_of_unity: *mut BlstFr
}

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
pub struct Poly {
    pub coeffs: *mut BlstFr,
    pub length: u64
}

#[link(name = "ckzg")]
#[link(name = "blst")]
extern "C" {
    pub fn new_poly(out: *mut Poly, length: u64) -> KzgRet;
    pub fn free_poly(p: *mut Poly);
}

#[cfg(test)]
mod tests {
    use crate::{Poly, new_poly, free_poly, KzgRet, BlstFr, FFTSettings};
    use crate::fftsettings::{ckzg_new_fft_settings};

    #[test]
    fn test_poly() {
        unsafe {
            let some_l: [u64; 4] = [0, 0, 0, 0];
            let some_poly = &mut Poly{ coeffs: &mut BlstFr{l: some_l }, length: 4 };
            assert_eq!(new_poly(some_poly, 4), KzgRet::KzgOk);
            free_poly(some_poly);
        }
    }

    #[test]
    fn test_fft_settings_alloc() {
        let root_of_unity_poly: [u64; 4] = [0, 0, 0, 0];
        let root_of_unity = &mut BlstFr{l: root_of_unity_poly };
        let expanded_roots_of_unity_poly: [u64; 4] = [0, 0, 0, 0];
        let expanded_roots_of_unity = &mut BlstFr{l: expanded_roots_of_unity_poly };
        let reverse_roots_of_unity_poly: [u64; 4] = [0, 0, 0, 0];
        let reverse_roots_of_unity = &mut BlstFr{l: reverse_roots_of_unity_poly };
        let settings = &mut FFTSettings{
            max_width: 16,
            root_of_unity: root_of_unity,
            expanded_roots_of_unity: expanded_roots_of_unity,
            reverse_roots_of_unity: reverse_roots_of_unity
        };

        assert_eq!(ckzg_new_fft_settings(settings, 16), KzgRet::KzgOk);
        // no free needed here, allocation on stack
    }

}
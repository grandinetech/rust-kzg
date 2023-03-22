use crate::consts::{
    BlstFp, BlstP1, BlstP1Affine, BlstP2, BlstP2Affine, BLST_ERROR, G1_NEGATIVE_GENERATOR,
    G2_NEGATIVE_GENERATOR,
};

use kzg::{Fr, G1Mul, G2Mul, G1, G2};
use rand::{thread_rng, RngCore};

extern "C" {
    // typedef uint8_t byte;
    // typedef uint64_t limb_t;

    // typedef struct { limb_t l[256/8/sizeof(limb_t)]; } blst_fr;
    // typedef struct { limb_t l[384/8/sizeof(limb_t)]; } blst_fp;
    // /* 0 is "real" part, 1 is "imaginary" */
    // typedef struct { blst_fp fp[2]; } blst_fp2;
    // typedef struct { blst_fp2 fp2[3]; } blst_fp6;
    // typedef struct { blst_fp6 fp6[2]; } blst_fp12;

    // Fr
    fn fr_from_uint64(out: *mut BlstFr, n: u64);
    fn fr_from_uint64s(out: *mut BlstFr, vals: *const u64);
    fn fr_to_uint64s(out: *mut u64, fr: *const BlstFr);
    fn fr_is_zero(p: *const BlstFr) -> bool;
    fn fr_is_null(p: *const BlstFr) -> bool;
    fn fr_is_one(p: *const BlstFr) -> bool;
    fn fr_equal(aa: *const BlstFr, bb: *const BlstFr) -> bool;
    fn fr_negate(out: *mut BlstFr, in_: *const BlstFr);
    fn fr_pow(out: *mut BlstFr, a: *const BlstFr, n: u64);
    fn fr_div(out: *mut BlstFr, a: *const BlstFr, b: *const BlstFr);
    fn blst_fr_add(ret: *mut BlstFr, a: *const BlstFr, b: *const BlstFr);
    fn blst_fr_sub(ret: *mut BlstFr, a: *const BlstFr, b: *const BlstFr);
    fn blst_fr_sqr(ret: *mut BlstFr, a: *const BlstFr);
    fn blst_fr_mul(ret: *mut BlstFr, a: *const BlstFr, b: *const BlstFr);
    fn blst_fr_eucl_inverse(ret: *mut BlstFr, a: *const BlstFr);
    fn blst_fr_inverse(retret: *mut BlstFr, a: *const BlstFr);
    // G1
    fn blst_p1_generator() -> *const BlstP1;
    fn g1_add_or_dbl(out: *mut BlstP1, a: *const BlstP1, b: *const BlstP1);
    fn g1_equal(a: *const BlstP1, b: *const BlstP1) -> bool;
    fn g1_mul(out: *mut BlstP1, a: *const BlstP1, b: *const BlstFr);
    fn g1_dbl(out: *mut BlstP1, a: *const BlstP1);
    fn g1_add(out: *mut BlstP1, a: *const BlstP1, b: *const BlstP1);
    fn g1_sub(out: *mut BlstP1, a: *const BlstP1, b: *const BlstP1);
    fn g1_is_inf(a: *const BlstP1) -> bool;
    pub fn blst_p1_from_affine(out: *mut BlstP1, inp: *const BlstP1Affine);
    pub fn blst_p1_compress(out: *mut u8, inp: *const BlstP1);
    pub fn blst_p1_uncompress(out: *mut BlstP1Affine, byte: *const u8) -> BLST_ERROR;
    // G2
    fn blst_p2_generator() -> *const BlstP2;
    fn g2_mul(out: *mut BlstP2, a: *const BlstP2, b: *const BlstFr);
    fn g2_dbl(out: *mut BlstP2, a: *const BlstP2);
    fn g2_add_or_dbl(out: *mut BlstP2, a: *const BlstP2, b: *const BlstP2);
    fn g2_equal(a: *const BlstP2, b: *const BlstP2) -> bool;
    fn g2_sub(out: *mut BlstP2, a: *const BlstP2, b: *const BlstP2);
    pub fn blst_p2_from_affine(out: *mut BlstP2, inp: *const BlstP2Affine);
    pub fn blst_p2_uncompress(out: *mut BlstP2Affine, byte: *const u8) -> BLST_ERROR;
    // Regular functions
    pub fn g1_linear_combination(
        out: *mut BlstP1,
        p: *const BlstP1,
        coeffs: *const BlstFr,
        len: u64,
    );
    fn pairings_verify(
        a1: *const BlstP1,
        a2: *const BlstP2,
        b1: *const BlstP1,
        b2: *const BlstP2,
    ) -> bool;
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct BlstFr {
    pub l: [u64; 4],
}

impl Fr for BlstFr {
    fn null() -> Self {
        Self { l: [u64::MAX; 4] }
    }

    fn zero() -> Self {
        Fr::from_u64(0)
    }

    fn one() -> Self {
        Fr::from_u64(1)
    }

    fn rand() -> Self {
        let mut ret = Self::default();
        let mut rng = thread_rng();
        let a: [u64; 4] = [
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
        ];
        unsafe {
            fr_from_uint64s(&mut ret, a.as_ptr());
        }
        ret
    }

    fn from_u64_arr(u: &[u64; 4]) -> Self {
        let mut ret = Self::default();
        unsafe {
            fr_from_uint64s(&mut ret, u.as_ptr());
        }
        ret
    }

    fn from_u64(u: u64) -> Self {
        let mut fr = Self::default();
        unsafe {
            fr_from_uint64(&mut fr, u);
        }
        fr
    }

    fn to_u64_arr(&self) -> [u64; 4] {
        let mut arr: [u64; 4] = [0; 4];
        unsafe {
            fr_to_uint64s(arr.as_mut_ptr(), self);
        }
        arr
    }

    fn is_one(&self) -> bool {
        unsafe { fr_is_one(self) }
    }

    fn is_zero(&self) -> bool {
        unsafe { fr_is_zero(self) }
    }

    fn is_null(&self) -> bool {
        unsafe { fr_is_null(self) }
    }

    fn sqr(&self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_sqr(&mut ret, self);
        }
        ret
    }

    fn mul(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_mul(&mut ret, self, b);
        }
        ret
    }

    fn add(&self, b: &Self) -> Self {
        let mut sum = Self::default();
        unsafe {
            blst_fr_add(&mut sum, self, b);
        }
        sum
    }

    fn sub(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_sub(&mut ret, self, b);
        }
        ret
    }

    fn eucl_inverse(&self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_eucl_inverse(&mut ret, self);
        }

        ret
    }

    fn negate(&self) -> Self {
        let mut ret = Self::default();
        unsafe {
            // man rodos, kad tokios nera
            fr_negate(&mut ret, self);
        }
        ret
    }

    fn inverse(&self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_inverse(&mut ret, self);
        }

        ret
    }

    fn pow(&self, n: usize) -> Self {
        let mut ret = Self::default();
        unsafe {
            fr_pow(&mut ret, self, n as u64);
        }
        ret
    }

    fn div(&self, b: &Self) -> Result<Self, String> {
        let mut ret = Self::default();
        unsafe {
            fr_div(&mut ret, self, b);
        }
        Ok(ret)
    }

    fn equals(&self, b: &Self) -> bool {
        unsafe { fr_equal(self, b) }
    }
}

impl G1 for BlstP1 {
    fn identity() -> Self {
        Self {
            x: BlstFp { l: [0; 6] },
            y: BlstFp { l: [0; 6] },
            z: BlstFp { l: [0; 6] },
        }
    }

    fn generator() -> Self {
        unsafe { *blst_p1_generator() }
    }

    fn negative_generator() -> Self {
        G1_NEGATIVE_GENERATOR
    }

    fn rand() -> Self {
        let mut ret = BlstP1::default();
        let random = Fr::rand();
        unsafe {
            g1_mul(&mut ret, &G1::generator(), &random);
        }
        ret
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        let mut out = BlstP1::default();
        unsafe {
            g1_add_or_dbl(&mut out, b, self);
        }
        out
    }

    fn is_inf(&self) -> bool {
        unsafe { g1_is_inf(self) }
    }

    fn dbl(&self) -> Self {
        let mut ret = BlstP1::default();
        unsafe {
            g1_dbl(&mut ret, self);
        }
        ret
    }

    fn add(&self, b: &Self) -> Self {
        let mut ret = BlstP1::default();
        unsafe {
            g1_add(&mut ret, self, b);
        }
        ret
    }

    fn sub(&self, b: &Self) -> Self {
        let mut ret = BlstP1::default();
        unsafe {
            g1_sub(&mut ret, self, b);
        }
        ret
    }

    fn equals(&self, b: &Self) -> bool {
        unsafe { g1_equal(self, b) }
    }
}

impl G1Mul<BlstFr> for BlstP1 {
    fn mul(&self, b: &BlstFr) -> Self {
        let mut ret = BlstP1::default();
        unsafe {
            g1_mul(&mut ret, self, b);
        }
        ret
    }
}

impl G2 for BlstP2 {
    fn generator() -> Self {
        unsafe { *blst_p2_generator() }
    }

    fn negative_generator() -> Self {
        G2_NEGATIVE_GENERATOR
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        let mut ret = BlstP2::default();
        unsafe {
            g2_add_or_dbl(&mut ret, self, b);
        }
        ret
    }

    fn dbl(&self) -> Self {
        let mut ret = BlstP2::default();
        unsafe {
            g2_dbl(&mut ret, self);
        }
        ret
    }

    fn sub(&self, b: &Self) -> Self {
        let mut ret = BlstP2::default();
        unsafe {
            g2_sub(&mut ret, self, b);
        }
        ret
    }

    fn equals(&self, b: &Self) -> bool {
        unsafe { g2_equal(self, b) }
    }
}

impl G2Mul<BlstFr> for BlstP2 {
    fn mul(&self, b: &BlstFr) -> Self {
        let mut ret = BlstP2::default();
        unsafe {
            g2_mul(&mut ret, self, b);
        }
        ret
    }
}

pub fn linear_combination_g1(out: &mut BlstP1, p: &[BlstP1], coeffs: &[BlstFr], len: usize) {
    unsafe {
        g1_linear_combination(out, p.as_ptr(), coeffs.as_ptr(), len as u64);
    }
}

pub fn verify_pairings(a1: &BlstP1, a2: &BlstP2, b1: &BlstP1, b2: &BlstP2) -> bool {
    unsafe { pairings_verify(a1, a2, b1, b2) }
}

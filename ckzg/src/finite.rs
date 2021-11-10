use kzg::{Fr, G1, G1Mul, G2, G2Mul};
use rand::{Rng, thread_rng};
use crate::consts::{BlstFp, BlstFp2, BlstP1, BlstP2, G1_NEGATIVE_GENERATOR, G2_NEGATIVE_GENERATOR};

extern "C" {
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
    fn blst_fr_sqr(ret: *mut BlstFr, a: *const BlstFr);
    fn blst_fr_mul(ret: *mut BlstFr, a: *const BlstFr, b: *const BlstFr);
    // G1
    fn blst_p1_generator() -> *const BlstP1;
    fn g1_add_or_dbl(out: *mut BlstP1, a: *const BlstP1, b: *const BlstP1);
    fn g1_equal(a: *const BlstP1, b: *const BlstP1) -> bool;
    fn g1_mul(out: *mut BlstP1, a: *const BlstP1, b: *const BlstFr);
    fn g1_dbl(out: *mut BlstP1, a: *const BlstP1);
    fn g1_sub(out: *mut BlstP1, a: *const BlstP1, b: *const BlstP1);
    fn g1_is_inf(a: *const BlstP1) -> bool;
    // G2
    fn blst_p2_generator() -> *const BlstP2;
    fn g2_mul(out: *mut BlstP2, a: *const BlstP2, b: *const BlstFr);
    fn g2_dbl(out: *mut BlstP2, a: *const BlstP2);
    fn g2_add_or_dbl(out: *mut BlstP2, a: *const BlstP2, b: *const BlstP2);
    fn g2_equal(a: *const BlstP2, b: *const BlstP2) -> bool;
    fn g2_sub(out: *mut BlstP2, a: *const BlstP2, b: *const BlstP2);
    // Regular functions
    fn g1_linear_combination(out: *mut BlstP1, p: *const BlstP1, coeffs: *const BlstFr, len: u64);
    fn pairings_verify(a1: *const BlstP1, a2: *const BlstP2, b1: *const BlstP1, b2: *const BlstP2) -> bool;
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct BlstFr {
    pub l: [u64; 4]
}

impl Fr for BlstFr {
    fn default() -> Self {
        Self { l: [0; 4] }
    }

    fn null() -> Self {
        Self { l: [u64::MAX; 4]}
    }

    fn zero() -> Self {
        Fr::from_u64(0)
    }

    fn one() -> Self {
        Fr::from_u64(1)
    }

    fn rand() -> Self {
        let mut ret = Fr::default();
        let mut rng = thread_rng();
        let a: [u64; 4] = [
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64()
        ];
        unsafe {
            fr_from_uint64s(&mut ret, a.as_ptr());
        }
        ret
    }

    fn from_u64_arr(u: &[u64; 4]) -> Self {
        let mut ret = Fr::default();
        unsafe {
            fr_from_uint64s(&mut ret, u.as_ptr());
        }
        ret
    }

    fn from_u64(u: u64) -> Self {
        let mut fr = Fr::default();
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
        unsafe {
            return fr_is_one(self);
        }
    }

    fn is_zero(&self) -> bool {
        unsafe {
            return fr_is_zero(self);
        }
    }

    fn is_null(&self) -> bool {
        unsafe {
            return fr_is_null(self);
        }
    }

    fn sqr(&self) -> Self {
        let mut ret = Fr::default();
        unsafe {
            blst_fr_sqr(&mut ret, self);
        }
        ret
    }

    fn mul(&self, b: &Self) -> Self {
        let mut ret = Fr::default();
        unsafe {
            blst_fr_mul(&mut ret, self, b);
        }
        ret
    }

    fn add(&self, b: &Self) -> Self {
        let mut sum = Fr::default();
        unsafe {
            blst_fr_add(&mut sum, self, b);
        }
        sum
    }

    fn sub(&self, _b: &Self) -> Self {
        todo!()
    }

    fn eucl_inverse(&self) -> Self {
        todo!()
    }

    fn negate(&self) -> Self {
        let mut ret = Fr::default();
        unsafe {
            fr_negate(&mut ret, self);
        }
        ret
    }

    fn inverse(&self) -> Self {
        todo!()
    }

    fn pow(&self, n: usize) -> Self {
        let mut ret = Fr::default();
        unsafe {
            fr_pow(&mut ret, self, n as u64);
        }
        ret
    }

    fn div(&self, b: &Self) -> Result<Self, String> {
        let mut ret = Fr::default();
        unsafe {
            fr_div(&mut ret, self, b);
        }
        Ok(ret)
    }

    fn equals(&self, b: &Self) -> bool {
        unsafe {
            return fr_equal(self, b);
        }
    }

    fn destroy(&mut self) {}
}

impl G1 for BlstP1 {
    fn default() -> Self {
        Self {
            x: BlstFp { l: [0; 6] },
            y: BlstFp { l: [0; 6] },
            z: BlstFp { l: [0; 6] },
        }
    }

    fn identity() -> Self {
        Self {
            x: BlstFp { l: [0; 6] },
            y: BlstFp { l: [0; 6] },
            z: BlstFp { l: [0; 6] },
        }
    }

    fn generator() -> Self {
        unsafe {
            return *blst_p1_generator();
        }
    }

    fn negative_generator() -> Self {
        G1_NEGATIVE_GENERATOR
    }

    fn rand() -> Self {
        let mut ret = G1::default();
        let random = Fr::rand();
        unsafe {
            g1_mul(&mut ret, &G1::generator(), &random);
        }
        ret
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        let mut out = G1::default();
        unsafe {
            g1_add_or_dbl(&mut out, b, self);
        }
        out
    }

    fn is_inf(&self) -> bool {
        unsafe {
            return g1_is_inf(self);
        }
    }

    fn dbl(&self) -> Self {
        let mut ret = G1::default();
        unsafe {
            g1_dbl(&mut ret, self);
        }
        ret
    }

    fn sub(&self, b: &Self) -> Self {
        let mut ret = G1::default();
        unsafe {
            g1_sub(&mut ret, self, b);
        }
        ret
    }

    fn equals(&self, b: &Self) -> bool {
        unsafe {
            return g1_equal(self, b);
        }
    }

    fn destroy(&mut self) {}
}

impl G1Mul<BlstFr> for BlstP1 {
    fn mul(&self, b: &BlstFr) -> Self {
        let mut ret = G1::default();
        unsafe {
            g1_mul(&mut ret, self, b);
        }
        ret
    }
}

impl G2 for BlstP2 {
    fn default() -> Self {
        Self {
            x: BlstFp2 { fp: [BlstFp { l: [0; 6] }, BlstFp { l: [0; 6] }] },
            y: BlstFp2 { fp: [BlstFp { l: [0; 6] }, BlstFp { l: [0; 6] }] },
            z: BlstFp2 { fp: [BlstFp { l: [0; 6] }, BlstFp { l: [0; 6] }] },
        }
    }

    fn generator() -> Self {
        unsafe {
            return *blst_p2_generator();
        }
    }

    fn negative_generator() -> Self {
        G2_NEGATIVE_GENERATOR
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        let mut ret = G2::default();
        unsafe {
            g2_add_or_dbl(&mut ret, self, b);
        }
        ret
    }

    fn dbl(&self) -> Self {
        let mut ret = G2::default();
        unsafe {
            g2_dbl(&mut ret, self);
        }
        ret
    }

    fn sub(&self, b: &Self) -> Self {
        let mut ret = G2::default();
        unsafe {
            g2_sub(&mut ret, self, b);
        }
        ret
    }

    fn equals(&self, b: &Self) -> bool {
        unsafe {
            return g2_equal(self, b);
        }
    }

    fn destroy(&mut self) {}
}

impl G2Mul<BlstFr> for BlstP2 {
    fn mul(&self, b: &BlstFr) -> Self {
        let mut ret = G2::default();
        unsafe {
            g2_mul(&mut ret, self, b);
        }
        ret
    }
}

pub fn linear_combination_g1(out: &mut BlstP1, p: &Vec<BlstP1>, coeffs: &Vec<BlstFr>, len: usize) {
    unsafe {
        g1_linear_combination(out, p.as_ptr(), coeffs.as_ptr(), len as u64);
    }
}

pub fn verify_pairings(a1: &BlstP1, a2: &BlstP2, b1: &BlstP1, b2: &BlstP2) -> bool {
    unsafe {
        return pairings_verify(a1, a2, b1, b2);
    }
}

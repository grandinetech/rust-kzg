use kzg::{Fr, G1, G1Mul, G2, G2Mul};
use rand::{Rng, thread_rng};
use crate::consts::{BlstFp, BlstFp2, BlstP1, BlstP2};

extern "C" {
    // Fr
    fn fr_from_uint64(out: *mut BlstFr, n: u64);
    fn fr_from_uint64s(out: *mut BlstFr, vals: *const u64);
    fn fr_is_zero(p: *const BlstFr) -> bool;
    fn fr_is_one(p: *const BlstFr) -> bool;
    fn fr_equal(aa: *const BlstFr, bb: *const BlstFr) -> bool;
    fn fr_negate(out: *mut BlstFr, in_: *const BlstFr);
    //fn fr_pow(out: *mut BlstFr, a: *const BlstFr, n: u64);
    fn blst_fr_add(ret: *mut BlstFr, a: *const BlstFr, b: *const BlstFr);
    fn blst_fr_sqr(ret: *mut BlstFr, a: *const BlstFr);
    fn blst_fr_mul(ret: *mut BlstFr, a: *const BlstFr, b: *const BlstFr);
    fn blst_fr_from_uint64(ret: *mut BlstFr, a: *const u64);
    // G1
    fn blst_p1_generator() -> *const BlstP1;
    static g1_negative_generator: BlstP1;
    fn g1_add_or_dbl(out: *mut BlstP1, a: *const BlstP1, b: *const BlstP1);
    fn g1_equal(a: *const BlstP1, b: *const BlstP1) -> bool;
    fn g1_mul(out: *mut BlstP1, a: *const BlstP1, b: *const BlstFr);
    // G2
    fn blst_p2_generator() -> *const BlstP2;
    static g2_negative_generator: BlstP2;
    fn g2_mul(out: *mut BlstP2, a: *const BlstP2, b: *const BlstFr);
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
            blst_fr_from_uint64(&mut ret, u.as_ptr());
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
        todo!()
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

    fn pow(&self, _n: usize) -> Self {
        todo!()
    }

    fn div(&self, b: &Self) -> Result<Self, String> {
        todo!()
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
            x: BlstFp { l: [1; 6] },
            y: BlstFp { l: [1; 6] },
            z: BlstFp { l: [1; 6] },
        }
    }

    fn generator() -> Self {
        unsafe {
            return *blst_p1_generator();
        }
    }

    fn negative_generator() -> Self {
        unsafe {
            g1_negative_generator
        }
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
        let out = self;
        unsafe {
            g1_add_or_dbl(out, b, &G1::generator());
        }
        *out
    }

    fn is_inf(&self) -> bool {
        todo!()
    }

    fn dbl(&self) -> Self {
        todo!()
    }

    fn sub(&self, b: &Self) -> Self {
        todo!()
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
        unsafe {
            g2_negative_generator
        }
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        todo!()
    }

    fn dbl(&self) -> Self {
        todo!()
    }

    fn sub(&self, b: &Self) -> Self {
        todo!()
    }

    fn equals(&self, b: &Self) -> bool {
        todo!()
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

use rand::{Rng, thread_rng};
use kzg::Fr;

#[link(name = "blst", kind = "static")]
extern "C" {
    fn blst_fr_add(ret: *mut BlstFr, a: *const BlstFr, b: *const BlstFr);
    fn fr_from_uint64(out: *mut BlstFr, n: u64);
    fn fr_from_uint64s(out: *mut BlstFr, vals: *const u64);
    fn fr_is_zero(p: *const BlstFr) -> bool;
    fn fr_is_one(p: *const BlstFr) -> bool;
    fn fr_equal(aa: *const BlstFr, bb: *const BlstFr) -> bool;
    fn fr_negate(out: *mut BlstFr, in_: *const BlstFr);
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct BlstFr {
    pub l: [u64; 4]
}

impl Fr for BlstFr {
    fn default() -> Self {
        Self { l: [0, 0, 0, 0] }
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
        todo!()
    }

    fn from_u64(u: u64) -> Self {
        let mut fr = Fr::default();
        unsafe {
            fr_from_uint64(&mut fr, u);
        }
        fr
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
        todo!()
    }

    fn mul(&self, b: &Self) -> Self {
        todo!()
    }

    fn add(&self, b: &Self) -> Self {
        let mut sum = Fr::default();
        unsafe {
            blst_fr_add(&mut sum, self, b);
        }
        sum
    }

    fn sub(&self, b: &Self) -> Self {
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

    fn equals(&self, b: &Self) -> bool {
        unsafe {
            return fr_equal(self, b);
        }
    }
}

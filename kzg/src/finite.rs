use super::Fr;
use rand::{thread_rng, Rng};

#[link(name = "blst", kind = "static")]
extern "C" {
    fn blst_fr_add(ret: *mut Fr, a: *const Fr, b: *const Fr);
    fn fr_from_uint64(out: *mut Fr, n: u64);
    fn fr_from_uint64s(out: *mut Fr, vals: *const u64);
    fn fr_is_zero(p: *const Fr) -> bool;
    fn fr_is_one(p: *const Fr) -> bool;
    fn fr_equal(aa: *const Fr, bb: *const Fr) -> bool;
    fn fr_negate(out: *mut Fr, in_: *const Fr);
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct BlstFr {
    pub l: [u64; 4]
}

impl Fr {
    pub fn zero() -> Fr {
        Fr::from_u64(0)
    }

    pub fn one() -> Fr {
        Fr::from_u64(1)
    }

    pub fn rand() -> Fr {
        let mut ret: Fr = Fr::default();
        let mut rng = thread_rng();
        let a: [u64; 4] = [
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64()
        ];
        unsafe { fr_from_uint64s(&mut ret, a.as_ptr()); }
        ret
    }

    pub fn add(first: Fr, second: Fr) -> Fr {
        let mut sum = Fr::default();
        unsafe {
            blst_fr_add(&mut sum, &first, &second);
        }
        sum
    }

    pub fn from_u64(n: u64) -> Fr {
        let mut fr = Fr::default();
        unsafe {
            fr_from_uint64(&mut fr, n);
        }
        fr
    }

    pub fn is_zero(&self) -> bool {
        unsafe {
            return fr_is_zero(self);
        }
    }

    pub fn is_one(&self) -> bool {
        unsafe {
            return fr_is_one(self);
        }
    }

    pub fn is_equal(first: Fr, second: Fr) -> bool {
        unsafe {
            return fr_equal(&first, &second);
        }
    }

    pub fn negate(first: *const Fr) -> Fr {
        let mut ret: Fr = Fr::default();
        unsafe {
            fr_negate(&mut ret, first);
        }
        ret
    }
}

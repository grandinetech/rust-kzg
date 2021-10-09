use rand::{thread_rng, Rng};
use super::Fr;

#[link(name = "blst", kind = "static")]
extern "C" {
    fn blst_fr_add(ret: *mut Fr, a: *const Fr, b: *const Fr);
    fn fr_from_uint64(out: *mut Fr, n: u64);
    fn fr_from_uint64s(out: *mut Fr, vals: *const u64);
    fn fr_is_zero(p: *const Fr) -> bool;
    fn fr_equal(aa: *const Fr, bb: *const Fr) -> bool;
}

pub fn rand_fr() -> Fr {
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

pub fn add_fr(first: Fr, second: Fr) -> Fr {
    let mut sum = Fr::default();
    unsafe {
        blst_fr_add(&mut sum, &first, &second);
    }
    sum
}

pub fn u64_to_fr(n: u64) -> Fr {
    let mut fr = Fr::default();
    unsafe {
        fr_from_uint64(&mut fr, n);
    }
    fr
}

pub fn is_zero_fr(point: Fr) -> bool {
    unsafe {
        return fr_is_zero(&point);
    }
}

pub fn is_equal(first: Fr, second: Fr) -> bool {
    unsafe {
        return fr_equal(&first, &second);
    }
}

pub fn zero_fr() -> Fr {
    u64_to_fr(0)
}

pub fn one_fr() -> Fr {
    u64_to_fr(1)
}

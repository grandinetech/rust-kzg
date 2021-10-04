use super::Fr;

#[link(name = "blst", kind = "static")]
extern "C" {
    fn blst_fr_add(ret: *mut Fr, a: *const Fr, b: *const Fr);
}

pub fn add_fr(first: Fr, second: Fr) -> Fr {
    let mut sum = Fr::default();
    unsafe {
        blst_fr_add(&mut sum, &first, &second);
    }
    sum
}

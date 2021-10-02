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

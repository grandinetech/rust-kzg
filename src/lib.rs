pub type KzgRet = ::std::os::raw::c_uint;

pub const KZG_OK: KzgRet = 0;
pub const KZG_BAD_ARGS: KzgRet = 1;
pub const KZG_ERROR: KzgRet = 2;
pub const KZG_MALLOC: KzgRet = 3;

/* TODO: need something like this:
enum KzgRet : uint_t {
    KZG_OK,
    KZG_BAD_ARGS,
    KZG_ERROR,
    KZG_MALLOC
};
*/

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
    use crate::{Poly, new_poly, free_poly, KZG_OK, BlstFr};

    #[test]
    fn it_works() {
        unsafe {
            let some_l: [u64; 4] = [0, 0, 0, 0];
            let some_poly = &mut Poly{ coeffs: &mut BlstFr{l: some_l }, length: 4 };
            assert_eq!(new_poly(some_poly, 4), KZG_OK);
            free_poly(some_poly);
        }
        assert_eq!(2 + 2, 4);
    }
}

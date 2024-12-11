extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use blst::blst_fr;

use crate::mcl_methods::mcl_fr;

use kzg::eip_4844::BYTES_PER_FIELD_ELEMENT;
use kzg::Fr;
use kzg::Scalar256;

#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct FsFr(pub mcl_fr);

impl Fr for FsFr {
    fn null() -> Self {
        todo!()
    }

    fn zero() -> Self {
        todo!()
    }

    fn one() -> Self {
        todo!()
    }

    #[cfg(feature = "rand")]
    fn rand() -> Self {
        use blst::blst_fr_from_uint64;

        let val: [u64; 4] = [
            rand::random(),
            rand::random(),
            rand::random(),
            rand::random(),
        ];

        let ret = Self::default();
        let mut blst = FsFr::to_blst_fr(&ret);

        unsafe {
            blst_fr_from_uint64(&mut blst, val.as_ptr());
        }

        FsFr::from_blst_fr(blst)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        todo!()
    }

    fn from_hex(hex: &str) -> Result<Self, String> {
        todo!()
    }

    fn from_u64_arr(u: &[u64; 4]) -> Self {
        todo!()
    }

    fn from_u64(u: u64) -> Self {
        todo!()
    }

    fn to_bytes(&self) -> [u8; 32] {
        todo!()
    }

    fn to_u64_arr(&self) -> [u64; 4] {
        todo!()
    }

    fn is_one(&self) -> bool {
        todo!()
    }

    fn is_zero(&self) -> bool {
        todo!()
    }

    fn is_null(&self) -> bool {
        todo!()
    }

    fn sqr(&self) -> Self {
        todo!()
    }

    fn mul(&self, b: &Self) -> Self {
        todo!()
    }

    fn add(&self, b: &Self) -> Self {
        todo!()
    }

    fn sub(&self, b: &Self) -> Self {
        todo!()
    }

    fn eucl_inverse(&self) -> Self {
        todo!()
    }

    fn negate(&self) -> Self {
        todo!()
    }

    fn inverse(&self) -> Self {
        todo!()
    }

    fn pow(&self, n: usize) -> Self {
        todo!()
    }

    fn div(&self, b: &Self) -> Result<Self, String> {
        todo!()
    }

    fn equals(&self, b: &Self) -> bool {
        todo!()
    }

    fn to_scalar(&self) -> Scalar256 {
        todo!()
    }
}

impl FsFr {
    pub fn from_blst_fr(fr: blst_fr) -> Self {
        Self {
            0: mcl_fr {
                d: fr.l
            }
        }
    }

    pub fn to_blst_fr(&self) -> blst_fr {
        blst_fr {
            l: self.0.d
        }
    }
}
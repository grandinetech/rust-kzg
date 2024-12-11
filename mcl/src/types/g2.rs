extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;

use kzg::eip_4844::BYTES_PER_G2;
#[cfg(feature = "rand")]
use kzg::Fr;
use kzg::{G2Mul, G2};

use crate::consts::{G2_GENERATOR, G2_NEGATIVE_GENERATOR};
use crate::mcl_methods::mcl_g2;
use crate::types::fr::FsFr;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct FsG2(pub mcl_g2);

impl FsG2 {
    pub const fn from_blst_p2(p2: blst::blst_p2) -> Self {
        todo!()
        // Self(blst_p2_into_pc_g2projective(&p2))
    }

    pub const fn to_blst_p2(&self) -> blst::blst_p2 {
        todo!()
        // pc_g2projective_into_blst_p2(self.0)
    }
}

impl G2 for FsG2 {
    fn generator() -> Self {
        todo!()
    }

    fn negative_generator() -> Self {
        todo!()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        todo!()
    }

    fn to_bytes(&self) -> [u8; 96] {
        todo!()
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
}

impl G2Mul<FsFr> for FsG2 {
    fn mul(&self, b: &FsFr) -> Self {
        todo!()
    }
}

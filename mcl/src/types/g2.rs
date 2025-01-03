extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;

use kzg::eip_4844::BYTES_PER_G2;
#[cfg(feature = "rand")]
use kzg::Fr;
use kzg::{G2Mul, G2};

use crate::consts::{G2_GENERATOR, G2_NEGATIVE_GENERATOR};
use crate::mcl_methods::mcl_fp;
use crate::mcl_methods::mcl_fp2;
use crate::mcl_methods::mcl_g2;
use crate::mcl_methods::try_init_mcl;
use crate::types::fr::FsFr;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct FsG2(pub mcl_g2);

impl FsG2 {
    pub fn from_blst_p2(p2: blst::blst_p2) -> Self {
        try_init_mcl();

        Self(mcl_g2 { 
            x: mcl_fp2{ d: [mcl_fp{ d: p2.x.fp[0].l }, mcl_fp{ d: p2.x.fp[1].l }] }, 
            y: mcl_fp2{ d: [mcl_fp{ d: p2.y.fp[0].l }, mcl_fp{ d: p2.y.fp[1].l }] }, 
            z: mcl_fp2{ d: [mcl_fp{ d: p2.z.fp[0].l }, mcl_fp{ d: p2.z.fp[1].l }] }, 
        })
        // Self(blst_p2_into_pc_g2projective(&p2))
    }

    pub const fn to_blst_p2(&self) -> blst::blst_p2 {
        blst::blst_p2 {
            x: blst::blst_fp2{ fp: [ blst::blst_fp{ l: self.0.x.d[0].d }, blst::blst_fp{ l: self.0.x.d[1].d } ] },
            y: blst::blst_fp2{ fp: [ blst::blst_fp{ l: self.0.y.d[0].d }, blst::blst_fp{ l: self.0.y.d[1].d } ] },
            z: blst::blst_fp2{ fp: [ blst::blst_fp{ l: self.0.z.d[0].d }, blst::blst_fp{ l: self.0.z.d[1].d } ] }
        }

        // pc_g2projective_into_blst_p2(self.0)
    }

    #[cfg(feature = "rand")]
    pub fn rand() -> Self {
        try_init_mcl();

        let result: FsG2 = G2_GENERATOR;
        result.mul(&FsFr::rand())
    }
}

impl G2 for FsG2 {
    fn generator() -> Self {
        try_init_mcl();

        G2_GENERATOR
    }

    fn negative_generator() -> Self {
        try_init_mcl();

        G2_NEGATIVE_GENERATOR
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        try_init_mcl();

        bytes
            .try_into()
            .map_err(|_| {
                format!(
                    "Invalid byte length. Expected {}, got {}",
                    BYTES_PER_G2,
                    bytes.len()
                )
            })
            .and_then(|bytes: &[u8; BYTES_PER_G2]| {
                use blst::{blst_p2_affine, blst_p2};

                let mut tmp = blst_p2_affine::default();
                let mut g2 = blst_p2::default();
                unsafe {
                    // The uncompress routine also checks that the point is on the curve
                    if blst::blst_p2_uncompress(&mut tmp, bytes.as_ptr()) != blst::BLST_ERROR::BLST_SUCCESS {
                        return Err("Failed to uncompress".to_string());
                    }
                    blst::blst_p2_from_affine(&mut g2, &tmp);
                }
                Ok(FsG2::from_blst_p2(g2))
            })
    }

    fn to_bytes(&self) -> [u8; 96] {
        todo!()
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        try_init_mcl();

        let mut out: mcl_g2 = mcl_g2::default();
        mcl_g2::add(&mut out, &self.0, &b.0);
        Self(out)
    }

    fn dbl(&self) -> Self {
        try_init_mcl();

        let mut out = mcl_g2::default();
        mcl_g2::dbl(&mut out, &self.0);
        Self(out)
    }

    fn sub(&self, b: &Self) -> Self {
        try_init_mcl();

        let mut out: mcl_g2 = mcl_g2::default();
        mcl_g2::sub(&mut out, &self.0, &b.0);
        Self(out)
    }

    fn equals(&self, b: &Self) -> bool {
        try_init_mcl();

        mcl_g2::eq(&self.0, &b.0)
    }
}

impl G2Mul<FsFr> for FsG2 {
    fn mul(&self, b: &FsFr) -> Self {
        try_init_mcl();

        let mut out: mcl_g2 = mcl_g2::default();
        mcl_g2::mul(&mut out, &self.0, &b.0);
        Self(out)
    }
}

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;

use blst::{
    blst_fp2, blst_p2, blst_p2_add_or_double, blst_p2_affine, blst_p2_cneg, blst_p2_compress,
    blst_p2_double, blst_p2_from_affine, blst_p2_is_equal, blst_p2_mult, blst_p2_uncompress,
    blst_scalar, blst_scalar_from_fr, BLST_ERROR,
};
use kzg::eip_4844::BYTES_PER_G2;
#[cfg(feature = "rand")]
use kzg::Fr;
use kzg::{G2Mul, G2};

use crate::consts::{G2_GENERATOR, G2_NEGATIVE_GENERATOR};
use crate::types::fr::FsFr;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct FsG2(pub blst_p2);

impl G2Mul<FsFr> for FsG2 {
    fn mul(&self, b: &FsFr) -> Self {
        let mut result = blst_p2::default();
        let mut scalar = blst_scalar::default();
        unsafe {
            blst_scalar_from_fr(&mut scalar, &b.0);
            blst_p2_mult(
                &mut result,
                &self.0,
                scalar.b.as_ptr(),
                8 * core::mem::size_of::<blst_scalar>(),
            );
        }
        Self(result)
    }
}

impl G2 for FsG2 {
    fn generator() -> Self {
        G2_GENERATOR
    }

    fn negative_generator() -> Self {
        G2_NEGATIVE_GENERATOR
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
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
                let mut tmp = blst_p2_affine::default();
                let mut g2 = blst_p2::default();
                unsafe {
                    // The uncompress routine also checks that the point is on the curve
                    if blst_p2_uncompress(&mut tmp, bytes.as_ptr()) != BLST_ERROR::BLST_SUCCESS {
                        return Err("Failed to uncompress".to_string());
                    }
                    blst_p2_from_affine(&mut g2, &tmp);
                }
                Ok(FsG2(g2))
            })
    }

    fn to_bytes(&self) -> [u8; 96] {
        let mut out = [0u8; BYTES_PER_G2];
        unsafe {
            blst_p2_compress(out.as_mut_ptr(), &self.0);
        }
        out
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        let mut result = blst_p2::default();
        unsafe {
            blst_p2_add_or_double(&mut result, &self.0, &b.0);
        }
        Self(result)
    }

    fn dbl(&self) -> Self {
        let mut result = blst_p2::default();
        unsafe {
            blst_p2_double(&mut result, &self.0);
        }
        Self(result)
    }

    fn sub(&self, b: &Self) -> Self {
        let mut bneg: blst_p2 = b.0;
        let mut result = blst_p2::default();
        unsafe {
            blst_p2_cneg(&mut bneg, true);
            blst_p2_add_or_double(&mut result, &self.0, &bneg);
        }
        Self(result)
    }

    fn equals(&self, b: &Self) -> bool {
        unsafe { blst_p2_is_equal(&self.0, &b.0) }
    }
}

impl FsG2 {
    pub(crate) fn _from_xyz(x: blst_fp2, y: blst_fp2, z: blst_fp2) -> Self {
        FsG2(blst_p2 { x, y, z })
    }

    #[cfg(feature = "rand")]
    pub fn rand() -> Self {
        let result: FsG2 = G2_GENERATOR;
        result.mul(&FsFr::rand())
    }
}

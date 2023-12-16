extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;

use kzg::eip_4844::BYTES_PER_G2;
#[cfg(feature = "rand")]
use kzg::Fr;
use kzg::{G2Mul, G2};

use crate::consts::{G2_GENERATOR, G2_NEGATIVE_GENERATOR};
use crate::types::fr::CtFr;

use constantine_sys::{bls12_381_g2_jac, bls12_381_g2_aff, ctt_bls12_381_g2_jac_cneg_in_place, bls12_381_fp2, ctt_bls12_381_fp2_double_in_place,
    ctt_bls12_381_g2_jac_from_affine, ctt_bls12_381_g1_jac_is_eq};

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct CtG2(pub bls12_381_g2_jac);

impl G2Mul<CtFr> for CtG2 {
    fn mul(&self, b: &CtFr) -> Self {
        let mut result = bls12_381_g2_jac::default();
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

impl G2 for CtG2 {
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
                let mut tmp = bls12_381_g2_aff::default();
                let mut g2 = bls12_381_g2_jac::default();
                unsafe {
                    // The uncompress routine also checks that the point is on the curve
                    if blst_p2_uncompress(&mut tmp, bytes.as_ptr()) != BLST_ERROR::BLST_SUCCESS {
                        return Err("Failed to uncompress".to_string());
                    }
                    ctt_bls12_381_g2_jac_from_affine(&mut g2, &tmp);
                }
                Ok(CtG2(g2))
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
        let mut result = bls12_381_g2_jac::default();
        unsafe {
            blst_p2_add_or_double(&mut result, &self.0, &b.0);
        }
        Self(result)
    }

    fn dbl(&self) -> Self {
        let mut result = bls12_381_g2_jac::default();
        unsafe {
            ctt_bls12_381_fp2_double_in_place(&mut result);
        }
        Self(result)
    }

    fn sub(&self, b: &Self) -> Self {
        let mut bneg: bls12_381_g2_jac = b.0;
        let mut result = bls12_381_g2_jac::default();
        unsafe {
            //blst_p2_cneg(&mut bneg, true);
            ctt_bls12_381_g2_jac_cneg_in_place(&mut bneg, true);
            blst_p2_add_or_double(&mut result, &self.0, &bneg);
        }
        Self(result)
    }

    fn equals(&self, b: &Self) -> bool {
        unsafe { ctt_bls12_381_g1_jac_is_eq(&self.0, &b.0) }
    }
}

impl CtG2 {
    pub(crate) fn _from_xyz(x: bls12_381_fp2, y: bls12_381_fp2, z: bls12_381_fp2) -> Self {
        CtG2(bls12_381_g2_jac { x, y, z })
    }

    #[cfg(feature = "rand")]
    pub fn rand() -> Self {
        let result: CtG2 = G2_GENERATOR;
        result.mul(&CtFr::rand())
    }
}

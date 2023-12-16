extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;

use kzg::common_utils::log_2_byte;
use kzg::eip_4844::BYTES_PER_G1;
use kzg::{G1Mul, G1};

use crate::consts::{G1_GENERATOR, G1_IDENTITY, G1_NEGATIVE_GENERATOR};
use crate::kzg_proofs::g1_linear_combination;
use crate::types::fr::CtFr;

use constantine_sys::{bls12_381_fp, bls12_381_g1_jac, bls12_381_g1_aff, ctt_bls12_381_g1_jac_double, ctt_bls12_381_g1_jac_sum, ctt_bls12_381_g1_jac_is_inf,
    ctt_bls12_381_g1_jac_is_eq, ctt_bls12_381_g1_jac_cneg_in_place, ctt_bls12_381_g1_jac_from_affine};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct CtG1(pub bls12_381_g1_jac);

impl CtG1 {
    pub(crate) const fn from_xyz(x: bls12_381_fp, y: bls12_381_fp, z: bls12_381_fp) -> Self {
        CtG1(bls12_381_g1_jac { x, y, z })
    }
}

impl G1 for CtG1 {
    fn identity() -> Self {
        G1_IDENTITY
    }

    fn generator() -> Self {
        G1_GENERATOR
    }

    fn negative_generator() -> Self {
        G1_NEGATIVE_GENERATOR
    }

    #[cfg(feature = "rand")]
    fn rand() -> Self {
        let result: CtG1 = G1_GENERATOR;
        result.mul(&kzg::Fr::rand())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        bytes
            .try_into()
            .map_err(|_| {
                format!(
                    "Invalid byte length. Expected {}, got {}",
                    BYTES_PER_G1,
                    bytes.len()
                )
            })
            .and_then(|bytes: &[u8; BYTES_PER_G1]| {
                let mut tmp = bls12_381_g1_aff::default();
                let mut g1 = bls12_381_g1_jac::default();
                unsafe {
                    // The uncompress routine also checks that the point is on the curve
                    if blst_p1_uncompress(&mut tmp, bytes.as_ptr()) != BLST_ERROR::BLST_SUCCESS {
                        return Err("Failed to uncompress".to_string());
                    }
                    ctt_bls12_381_g1_jac_from_affine(&mut g1, &tmp);
                }
                Ok(CtG1(g1))
            })
    }

    fn from_hex(hex: &str) -> Result<Self, String> {
        let bytes = hex::decode(&hex[2..]).unwrap();
        Self::from_bytes(&bytes)
    }

    fn to_bytes(&self) -> [u8; 48] {
        let mut out = [0u8; BYTES_PER_G1];
        unsafe {
            blst_p1_compress(out.as_mut_ptr(), &self.0);
        }
        out
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_p1_add_or_double(&mut ret.0, &self.0, &b.0);
        }
        ret
    }

    fn is_inf(&self) -> bool {
        unsafe { ctt_bls12_381_g1_jac_is_inf(&self.0) }
    }

    fn is_valid(&self) -> bool {
        unsafe {
            // The point must be on the right subgroup
            blst_p1_in_g1(&self.0)
        }
    }

    fn dbl(&self) -> Self {
        let mut result = bls12_381_g1_jac::default();
        unsafe {
            ctt_bls12_381_g1_jac_double(&mut result, &self.0);
        }
        Self(result)
    }

    fn add(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            ctt_bls12_381_g1_jac_sum(&mut ret.0, &self.0, &b.0);
        }
        ret
    }

    fn sub(&self, b: &Self) -> Self {
        let mut b_negative: CtG1 = *b;
        let mut ret = Self::default();
        unsafe {
            ctt_bls12_381_g1_jac_cneg_in_place(&mut b_negative.0, true);
            blst_p1_add_or_double(&mut ret.0, &self.0, &b_negative.0);
            ret
        }
    }

    fn equals(&self, b: &Self) -> bool {
        unsafe { ctt_bls12_381_g1_jac_is_eq(&self.0, &b.0) }
    }
}

impl G1Mul<CtFr> for CtG1 {
    fn mul(&self, b: &CtFr) -> Self {
        let mut scalar: blst_scalar = blst_scalar::default();
        unsafe {
            blst_scalar_from_fr(&mut scalar, &b.0);
        }

        // Count the number of bytes to be multiplied.
        let mut i = scalar.b.len();
        while i != 0 && scalar.b[i - 1] == 0 {
            i -= 1;
        }

        let mut result = Self::default();
        if i == 0 {
            return G1_IDENTITY;
        } else if i == 1 && scalar.b[0] == 1 {
            return *self;
        } else {
            // Count the number of bits to be multiplied.
            unsafe {
                blst_p1_mult(
                    &mut result.0,
                    &self.0,
                    &(scalar.b[0]),
                    8 * i - 7 + log_2_byte(scalar.b[i - 1]),
                );
            }
        }
        result
    }

    fn g1_lincomb(points: &[Self], scalars: &[CtFr], len: usize) -> Self {
        let mut out = CtG1::default();
        g1_linear_combination(&mut out, points, scalars, len);
        out
    }
}

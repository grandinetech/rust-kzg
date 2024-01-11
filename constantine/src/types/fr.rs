//blst_fp = bls12_381_fp, CtG1 = CtG1, blst_p1 = bls12_381_g1_jac, blst_fr = bls12_381_fr
extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;

use blst::blst_fr;
use constantine::ctt_codec_scalar_status;
use core::fmt::{Debug, Formatter};
use kzg::eip_4844::BYTES_PER_FIELD_ELEMENT;
use kzg::Fr;
use kzg::Scalar256;

use constantine_sys as constantine;

use constantine_sys::bls12_381_fr;

use crate::utils::ptr_transmute;
use crate::utils::ptr_transmute_mut;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CtFr(pub bls12_381_fr);

impl Debug for CtFr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "CtFr({:?})", self.0.limbs)
    }
}

impl PartialEq for CtFr {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}
impl Eq for CtFr {}

impl CtFr {
    pub fn from_blst_fr(fr: blst::blst_fr) -> Self {
        unsafe {
            Self(bls12_381_fr {
                limbs: core::mem::transmute(fr.l),
            })
        }
    }

    pub fn to_blst_fr(&self) -> blst_fr {
        unsafe {
            blst_fr {
                l: core::mem::transmute(self.0.limbs),
            }
        }
    }
}

impl Fr for CtFr {
    fn null() -> Self {
        Self::from_u64_arr(&[u64::MAX, u64::MAX, u64::MAX, u64::MAX])
    }

    fn zero() -> Self {
        Self::from_u64(0)
    }

    fn one() -> Self {
        Self::from_u64(1)
    }

    #[cfg(feature = "rand")]
    fn rand() -> Self {
        let val = constantine_sys::big255 {
            limbs: [
                rand::random(),
                rand::random(),
                rand::random(),
                rand::random(),
            ],
        };
        let mut ret = Self::default();
        unsafe {
            constantine::ctt_bls12_381_fr_from_big255(&mut ret.0, &val);
        }

        ret
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        bytes
            .try_into()
            .map_err(|_| {
                format!(
                    "Invalid byte length. Expected {}, got {}",
                    BYTES_PER_FIELD_ELEMENT,
                    bytes.len()
                )
            })
            .and_then(|bytes: &[u8; BYTES_PER_FIELD_ELEMENT]| {
                let mut ret: Self = Self::default();
                let mut scalar = constantine::big255::default();
                unsafe {
                    let status =
                        constantine::ctt_bls12_381_deserialize_scalar(&mut scalar, bytes.as_ptr());
                    if status == ctt_codec_scalar_status::cttCodecScalar_ScalarLargerThanCurveOrder
                    {
                        return Err("Invalid scalar".to_string());
                    }
                    constantine::ctt_bls12_381_fr_from_big255(&mut ret.0, &scalar);
                }
                Ok(ret)
            })
    }

    fn from_bytes_unchecked(bytes: &[u8]) -> Result<Self, String> {
        bytes
            .try_into()
            .map_err(|_| {
                format!(
                    "Invalid byte length. Expected {}, got {}",
                    BYTES_PER_FIELD_ELEMENT,
                    bytes.len()
                )
            })
            .map(|bytes: &[u8; BYTES_PER_FIELD_ELEMENT]| {
                let mut ret = Self::default();
                let mut scalar = constantine::big255::default();
                unsafe {
                    // FIXME: Seems like no 'non-validating' variant exists in constantine
                    blst::blst_scalar_from_bendian(ptr_transmute_mut(&mut scalar), bytes.as_ptr());
                    constantine::ctt_bls12_381_fr_from_big255(&mut ret.0, &scalar);
                }
                ret
            })
    }

    fn from_hex(hex: &str) -> Result<Self, String> {
        let bytes = hex::decode(&hex[2..]).unwrap();
        Self::from_bytes(&bytes)
    }

    fn from_u64_arr(u: &[u64; 4]) -> Self {
        let mut ret = Self::default();
        unsafe {
            constantine::ctt_bls12_381_fr_from_big255(&mut ret.0, ptr_transmute(u));
        }

        ret
    }

    fn from_u64(val: u64) -> Self {
        Self::from_u64_arr(&[val, 0, 0, 0])
    }

    fn to_bytes(&self) -> [u8; 32] {
        let mut scalar = constantine::big255::default();
        let mut bytes = [0u8; 32];
        unsafe {
            constantine::ctt_big255_from_bls12_381_fr(&mut scalar, &self.0);
            let _ = constantine::ctt_bls12_381_serialize_scalar(bytes.as_mut_ptr(), &scalar);
        }

        bytes
    }

    fn to_u64_arr(&self) -> [u64; 4] {
        let mut val: [u64; 4] = [0; 4];
        unsafe {
            constantine::ctt_big255_from_bls12_381_fr(ptr_transmute_mut(&mut val), &self.0);
        }

        val
    }

    fn is_one(&self) -> bool {
        unsafe { constantine::ctt_bls12_381_fr_is_one(&self.0) != 0 }
    }

    fn is_zero(&self) -> bool {
        unsafe { constantine::ctt_bls12_381_fr_is_zero(&self.0) != 0 }
    }

    fn is_null(&self) -> bool {
        self.equals(&Self::null())
    }

    fn sqr(&self) -> Self {
        let mut ret = Self::default();
        unsafe { constantine::ctt_bls12_381_fr_square(&mut ret.0, &self.0) }

        ret
    }

    fn mul(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            constantine::ctt_bls12_381_fr_prod(&mut ret.0, &self.0, &b.0);
        }

        ret
    }

    fn add(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            constantine::ctt_bls12_381_fr_sum(&mut ret.0, &self.0, &b.0);
        }

        ret
    }

    fn sub(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            constantine::ctt_bls12_381_fr_diff(&mut ret.0, &self.0, &b.0);
        }

        ret
    }

    fn eucl_inverse(&self) -> Self {
        let mut ret = Self::default();
        unsafe {
            constantine::ctt_bls12_381_fr_inv(&mut ret.0, &self.0);
        }

        ret
    }

    fn negate(&self) -> Self {
        let mut ret = *self;
        unsafe {
            constantine::ctt_bls12_381_fr_neg_in_place(&mut ret.0);
        }

        ret
    }

    fn inverse(&self) -> Self {
        let mut ret = Self::default();
        unsafe {
            constantine::ctt_bls12_381_fr_inv(&mut ret.0, &self.0);
        }

        ret
    }

    fn pow(&self, n: usize) -> Self {
        let mut out = Self::one();

        let mut temp = *self;
        let mut n = n;
        loop {
            if (n & 1) == 1 {
                out = out.mul(&temp);
            }
            n >>= 1;
            if n == 0 {
                break;
            }

            temp = temp.sqr();
        }

        out
    }

    fn div(&self, b: &Self) -> Result<Self, String> {
        let tmp = b.eucl_inverse();
        let out = self.mul(&tmp);

        Ok(out)
    }

    fn equals(&self, b: &Self) -> bool {
        unsafe { constantine::ctt_bls12_381_fr_is_eq(&self.0, &b.0) != 0 }
    }

    fn to_scalar(&self) -> kzg::Scalar256 {
        let mut scalar = constantine::big255::default();
        unsafe {
            constantine::ctt_big255_from_bls12_381_fr(&mut scalar, &self.0);
            Scalar256::from_u64(core::mem::transmute(scalar.limbs))
        }
    }
}

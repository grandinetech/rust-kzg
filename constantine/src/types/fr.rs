//blst_fp = bls12_381_fp, CtG1 = CtG1, blst_p1 = bls12_381_g1_jac, blst_fr = bls12_381_fr
extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;

use blst::blst_fr;
use kzg::eip_4844::BYTES_PER_FIELD_ELEMENT;
use kzg::Fr;
use kzg::Scalar256;

use constantine_sys as constantine;

use constantine_sys::{
    bls12_381_fr, ctt_bls12_381_fr_cneg_in_place, ctt_bls12_381_fr_diff, ctt_bls12_381_fr_inv,
    ctt_bls12_381_fr_prod, ctt_bls12_381_fr_square, ctt_bls12_381_fr_sum,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CtFr(pub bls12_381_fr);

impl CtFr {
    pub fn from_blst_fr(fr: blst::blst_fr) -> Self {
        Self(bls12_381_fr { limbs: fr.l })
    }

    pub fn to_blst_fr(&self) -> blst_fr {
        blst_fr { l: self.0.limbs }
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
        let val: [u64; 4] = [
            rand::random(),
            rand::random(),
            rand::random(),
            rand::random(),
        ];
        let mut ret = Self::default();
        unsafe {
            blst::blst_fr_from_uint64(core::mem::transmute(&mut ret.0), val.as_ptr());
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
                let mut bls_scalar = blst::blst_scalar::default();
                unsafe {
                    // FIXME: Change to constantine version when available
                    blst::blst_scalar_from_bendian(&mut bls_scalar, bytes.as_ptr());
                    if !blst::blst_scalar_fr_check(&bls_scalar) {
                        return Err("Invalid scalar".to_string());
                    }
                    blst::blst_fr_from_scalar(core::mem::transmute(&mut ret.0), &bls_scalar);
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
                let mut bls_scalar = blst::blst_scalar::default();
                unsafe {
                    blst::blst_scalar_from_bendian(&mut bls_scalar, bytes.as_ptr());
                    blst::blst_fr_from_scalar(core::mem::transmute(&mut ret.0), &bls_scalar);
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
            blst::blst_fr_from_uint64(core::mem::transmute(&mut ret.0), u.as_ptr());
        }

        ret
    }

    fn from_u64(val: u64) -> Self {
        Self::from_u64_arr(&[val, 0, 0, 0])
    }

    fn to_bytes(&self) -> [u8; 32] {
        let mut scalar = blst::blst_scalar::default();
        let mut bytes = [0u8; 32];
        unsafe {
            blst::blst_scalar_from_fr(&mut scalar, core::mem::transmute(&self.0));
            blst::blst_bendian_from_scalar(bytes.as_mut_ptr(), &scalar);
        }

        bytes
    }

    fn to_u64_arr(&self) -> [u64; 4] {
        let mut val: [u64; 4] = [0; 4];
        unsafe {
            blst::blst_uint64_from_fr(val.as_mut_ptr(), core::mem::transmute(&self.0));
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
        let mut val_a: [u64; 4] = [0; 4];
        let mut val_b: [u64; 4] = [0; 4];

        unsafe {
            constantine::ctt_bls12_381_fr_marshalBE(val_a.as_mut_ptr() as *mut u8, 32, &self.0);
            constantine::ctt_bls12_381_fr_marshalBE(val_b.as_mut_ptr() as *mut u8, 32, &b.0);
        }

        val_a[0] == val_b[0] && val_a[1] == val_b[1] && val_a[2] == val_b[2] && val_a[3] == val_b[3]
    }

    fn to_scalar(&self) -> kzg::Scalar256 {
        // FIXME: Change to constantine version when available
        let mut blst_scalar = blst::blst_scalar::default();
        unsafe {
            blst::blst_scalar_from_fr(&mut blst_scalar, core::mem::transmute(&self.0));
        }
        Scalar256::from_u8(&blst_scalar.b)
    }
}

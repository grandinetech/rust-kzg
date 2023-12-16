//blst_fp = bls12_381_fp, FsG1 = CtG1, blst_p1 = bls12_381_g1_jac, blst_fr = bls12_381_fr
extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;


use kzg::eip_4844::BYTES_PER_FIELD_ELEMENT;
use kzg::Fr;

use constantine_sys::{bls12_381_fr, ctt_bls12_381_fr_square, ctt_bls12_381_fr_sum, ctt_bls12_381_fr_diff, ctt_bls12_381_fr_prod, ctt_bls12_381_fr_inv, 
    ctt_bls12_381_fr_cneg_in_place};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct CtFr(pub bls12_381_fr);

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
            blst_fr_from_uint64(&mut ret.0, val.as_ptr());
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
                let mut bls_scalar = blst_scalar::default();
                let mut fr = bls12_381_fr::default();
                unsafe {
                    blst_scalar_from_bendian(&mut bls_scalar, bytes.as_ptr());
                    if !blst_scalar_fr_check(&bls_scalar) {
                        return Err("Invalid scalar".to_string());
                    }
                    blst_fr_from_scalar(&mut fr, &bls_scalar);
                }
                Ok(Self(fr))
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
                let mut bls_scalar = blst_scalar::default();
                let mut fr = bls12_381_fr::default();
                unsafe {
                    blst_scalar_from_bendian(&mut bls_scalar, bytes.as_ptr());
                    blst_fr_from_scalar(&mut fr, &bls_scalar);
                }
                Self(fr)
            })
    }

    fn from_hex(hex: &str) -> Result<Self, String> {
        let bytes = hex::decode(&hex[2..]).unwrap();
        Self::from_bytes(&bytes)
    }

    fn from_u64_arr(u: &[u64; 4]) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_from_uint64(&mut ret.0, u.as_ptr());
        }

        ret
    }

    fn from_u64(val: u64) -> Self {
        Self::from_u64_arr(&[val, 0, 0, 0])
    }

    fn to_bytes(&self) -> [u8; 32] {
        let mut scalar = blst_scalar::default();
        let mut bytes = [0u8; 32];
        unsafe {
            blst_scalar_from_fr(&mut scalar, &self.0);
            blst_bendian_from_scalar(bytes.as_mut_ptr(), &scalar);
        }

        bytes
    }

    fn to_u64_arr(&self) -> [u64; 4] {
        let mut val: [u64; 4] = [0; 4];
        unsafe {
            blst_uint64_from_fr(val.as_mut_ptr(), &self.0);
        }

        val
    }

    fn is_one(&self) -> bool {
        let mut val: [u64; 4] = [0; 4];
        unsafe {
            blst_uint64_from_fr(val.as_mut_ptr(), &self.0);
        }

        val[0] == 1 && val[1] == 0 && val[2] == 0 && val[3] == 0
    }

    fn is_zero(&self) -> bool {
        let mut val: [u64; 4] = [0; 4];
        unsafe {
            blst_uint64_from_fr(val.as_mut_ptr(), &self.0);
        }

        val[0] == 0 && val[1] == 0 && val[2] == 0 && val[3] == 0
    }

    fn is_null(&self) -> bool {
        self.equals(&Self::null())
    }

    fn sqr(&self) -> Self {
        let mut ret = Self::default();
        unsafe {
            ctt_bls12_381_fr_square(&mut ret.0, &self.0)
        }

        ret
    }

    fn mul(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            ctt_bls12_381_fr_prod(&mut ret.0, &self.0, &b.0);
        }

        ret
    }

    fn add(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            ctt_bls12_381_fr_sum(&mut ret.0, &self.0, &b.0);
        }

        ret
    }

    fn sub(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            ctt_bls12_381_fr_diff(&mut ret.0, &self.0, &b.0);
        }

        ret
    }

    fn eucl_inverse(&self) -> Self {
        let mut ret = Self::default();
        unsafe {
            ctt_bls12_381_fr_inv(&mut ret.0, &self.0);
        }

        ret
    }

    fn negate(&self) -> Self {
        let mut ret = Self::default();
        unsafe {
            ctt_bls12_381_fr_cneg_in_place(&mut ret.0, &self.0, true);
        }

        ret
    }

    fn inverse(&self) -> Self {
        let mut ret = Self::default();
        unsafe {
            ctt_bls12_381_fr_inv(&mut ret.0, &self.0);
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
            blst_uint64_from_fr(val_a.as_mut_ptr(), &self.0);
            blst_uint64_from_fr(val_b.as_mut_ptr(), &b.0);
        }

        val_a[0] == val_b[0] && val_a[1] == val_b[1] && val_a[2] == val_b[2] && val_a[3] == val_b[3]
    }
}

// Adds implementation for blsScalar (Fr)

pub use super::{BlsScalar, ZPoly};
use kzg::Fr;

use crate::curve::scalar::{sbb, Scalar, MODULUS, R2};
use kzg::eip_4844::BYTES_PER_FIELD_ELEMENT;
use std::convert::TryInto;

pub use crate::curve::scalar::Scalar as blsScalar;

impl Fr for blsScalar {
    fn null() -> Self {
        blsScalar::null()
    }
    fn zero() -> Self {
        blsScalar::zero()
    }

    fn one() -> Self {
        blsScalar::one()
    }

    fn rand() -> Self {
        let val: [u64; 4] = rand::random();
        blsScalar::from_raw(val)
    }

    #[allow(clippy::bind_instead_of_map)]
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
                let mut tmp = Scalar([0, 0, 0, 0]);

                tmp.0[0] = u64::from_le_bytes(<[u8; 8]>::try_from(&bytes[0..8]).unwrap());
                tmp.0[1] = u64::from_le_bytes(<[u8; 8]>::try_from(&bytes[8..16]).unwrap());
                tmp.0[2] = u64::from_le_bytes(<[u8; 8]>::try_from(&bytes[16..24]).unwrap());
                tmp.0[3] = u64::from_le_bytes(<[u8; 8]>::try_from(&bytes[24..32]).unwrap());

                // Try to subtract the modulus
                let (_, borrow) = sbb(tmp.0[0], MODULUS.0[0], 0);
                let (_, borrow) = sbb(tmp.0[1], MODULUS.0[1], borrow);
                let (_, borrow) = sbb(tmp.0[2], MODULUS.0[2], borrow);
                let (_, _borrow) = sbb(tmp.0[3], MODULUS.0[3], borrow);

                // Convert to Montgomery form by computing
                // (a.R^0 * R^2) / R = a.R
                tmp *= &R2;
                Ok(tmp)
            })
    }

    fn from_hex(hex: &str) -> Result<Self, String> {
        let bytes = hex::decode(&hex[2..]).unwrap();
        Fr::from_bytes(&bytes)
    }

    fn from_u64_arr(u: &[u64; 4]) -> Self {
        blsScalar::from_raw(*u)
    }

    fn from_u64(val: u64) -> Self {
        blsScalar::from(val)
    }

    fn to_bytes(&self) -> [u8; 32] {
        self.to_bytes()
    }

    fn to_u64_arr(&self) -> [u64; 4] {
        let bytes = self.to_bytes();
        [
            u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
            u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
            u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
            u64::from_le_bytes(bytes[24..32].try_into().unwrap()),
        ]
    }

    fn is_one(&self) -> bool {
        self.eq(&blsScalar::one())
    }

    fn is_zero(&self) -> bool {
        self.eq(&blsScalar::zero())
    }

    fn is_null(&self) -> bool {
        self.eq(&blsScalar::null())
    }
    fn sqr(&self) -> Self {
        blsScalar::square(self)
    }

    fn mul(&self, b: &Self) -> Self {
        blsScalar::mul(self, b)
    }

    fn add(&self, b: &Self) -> Self {
        blsScalar::add(self, b)
    }

    fn sub(&self, b: &Self) -> Self {
        blsScalar::sub(self, b)
    }

    fn eucl_inverse(&self) -> Self {
        self.invert().unwrap()
    }

    fn negate(&self) -> Self {
        self.neg()
    }

    fn inverse(&self) -> Self {
        self.invert().unwrap()
    }

    fn pow(&self, n: usize) -> Self {
        let mut tmp = *self;
        let mut out = Self::one();
        let mut n2 = n;

        loop {
            if n2 & 1 == 1 {
                out = out.mul(&tmp);
            }
            n2 >>= 1;
            if n2 == 0 {
                break;
            }
            tmp = tmp.sqr();
        }

        out
    }

    fn div(&self, b: &Self) -> Result<Self, String> {
        if <blsScalar as Fr>::is_zero(b) {
            return Ok(blsScalar::zero());
        }
        let tmp = b.eucl_inverse();
        let out = self.mul(&tmp);
        Ok(out)
    }

    fn equals(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

pub fn fr_div(a: &blsScalar, b: &blsScalar) -> Result<blsScalar, String> {
    if b.is_zero() {
        return Ok(blsScalar::zero());
    }
    let tmp = b.eucl_inverse();
    let out = a.mul(&tmp);
    Ok(out)
}

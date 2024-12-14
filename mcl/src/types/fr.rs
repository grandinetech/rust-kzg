extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use blst::blst_fr;
use blst::blst_fr_from_uint64;
use blst::blst_scalar;
use blst::blst_scalar_from_fr;
use blst::blst_uint64_from_fr;

use crate::mcl_methods::mcl_fr;
use crate::mcl_methods::try_init_mcl;

use kzg::eip_4844::BYTES_PER_FIELD_ELEMENT;
use kzg::Fr;
use kzg::Scalar256;

#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct FsFr(pub mcl_fr);

impl Fr for FsFr {
    fn null() -> Self {
        try_init_mcl();

        Self::from_u64_arr(&[u64::MAX, u64::MAX, u64::MAX, u64::MAX])
    }

    fn zero() -> Self {
        try_init_mcl();

        Self::from_u64(0)
    }

    fn one() -> Self {
        try_init_mcl();

        Self::from_u64(1)
    }

    #[cfg(feature = "rand")]
    fn rand() -> Self {
        try_init_mcl();

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

    fn from_u64_arr(val: &[u64; 4]) -> Self {
        try_init_mcl();

        let ret = Self::default();
        let mut blst = FsFr::to_blst_fr(&ret);
        
        unsafe {
            blst_fr_from_uint64(&mut blst, val.as_ptr());
        }

        FsFr::from_blst_fr(blst)
    }

    fn from_u64(val: u64) -> Self {
        try_init_mcl();
    
        Self::from_u64_arr(&[val, 0, 0, 0])
    }

    fn to_bytes(&self) -> [u8; 32] {
        todo!()
    }

    fn to_u64_arr(&self) -> [u64; 4] {
        try_init_mcl();

        let blst = self.to_blst_fr();
        
        let mut val: [u64; 4] = [0; 4];
        unsafe {
            blst_uint64_from_fr(val.as_mut_ptr(), &blst);
        }
 
        val
    }

    fn is_one(&self) -> bool {
        try_init_mcl();

        self.0.is_one()
    }

    fn is_zero(&self) -> bool {
        try_init_mcl();

        self.0.is_zero()
    }

    fn is_null(&self) -> bool {try_init_mcl();
        try_init_mcl();

        let n = Self::null();
        self.0.eq(&n.0)
    }

    fn sqr(&self) -> Self {
        try_init_mcl();

        let mut ret = Self::default();
        mcl_fr::sqr(&mut ret.0, &self.0);
        ret
    }

    fn mul(&self, b: &Self) -> Self {
        try_init_mcl();

        let mut ret = Self::default();
        mcl_fr::mul(&mut ret.0, &self.0, &b.0);
        ret
    }

    fn add(&self, b: &Self) -> Self {
        todo!()
    }

    fn sub(&self, b: &Self) -> Self {
        todo!()
    }

    fn eucl_inverse(&self) -> Self {
        try_init_mcl();

        let mut ret = Self::default();
        mcl_fr::inv(&mut ret.0, &self.0);
        ret
    }

    fn negate(&self) -> Self {
        try_init_mcl();

        let mut ret = Self::default();
        mcl_fr::neg(&mut ret.0, &self.0);
        ret
    }

    fn inverse(&self) -> Self {
        todo!()
    }

    fn pow(&self, n: usize) -> Self {
        try_init_mcl();

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
        try_init_mcl();

        if b.is_zero() {
            return Ok(*b)
        }

        let tmp = b.eucl_inverse();
        let out = self.mul(&tmp);

        Ok(out)
    }

    fn equals(&self, b: &Self) -> bool {
        try_init_mcl();

        mcl_fr::eq(&self.0, &b.0)
    }

    fn to_scalar(&self) -> Scalar256 {
        try_init_mcl();

        let blst = self.to_blst_fr();
        
        let mut blst_scalar = blst_scalar::default();
        unsafe {
            blst_scalar_from_fr(&mut blst_scalar, &blst);
        }
        Scalar256::from_u8(&blst_scalar.b)
    }
}

impl FsFr {
    pub fn from_blst_fr(fr: blst_fr) -> Self {
        try_init_mcl();

        Self {
            0: mcl_fr {
                d: fr.l
            }
        }
    }

    pub fn to_blst_fr(&self) -> blst_fr {
        try_init_mcl();

        blst_fr {
            l: self.0.d
        }
    }
}
use blst::{
    blst_fp, blst_p1, blst_p1_add_or_double, blst_p1_cneg, blst_p1_double, blst_p1_is_equal,
    blst_p1_is_inf, blst_p1_mult, blst_scalar, blst_scalar_from_fr,
};
use kzg::{Fr, G1Mul, G1};

use crate::consts::{G1_GENERATOR, G1_IDENTITY, G1_NEGATIVE_GENERATOR};
use crate::types::fr::FsFr;
use crate::utils::log_2_byte;

#[repr(C)]
pub struct FsG1(pub blst::blst_p1);

impl FsG1 {
    pub(crate) const fn from_xyz(x: blst_fp, y: blst_fp, z: blst_fp) -> Self {
        FsG1(blst_p1 { x, y, z })
    }
}

impl G1 for FsG1 {
    fn default() -> Self {
        Self(blst_p1::default())
    }

    fn identity() -> Self {
        G1_IDENTITY
    }

    fn generator() -> Self {
        G1_GENERATOR
    }

    fn negative_generator() -> Self {
        G1_NEGATIVE_GENERATOR
    }

    fn rand() -> Self {
        let result: FsG1 = G1_GENERATOR;
        result.mul(&FsFr::rand())
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_p1_add_or_double(&mut ret.0, &self.0, &b.0);
        }
        ret
    }

    fn is_inf(&self) -> bool {
        unsafe { blst_p1_is_inf(&self.0) }
    }

    fn dbl(&self) -> Self {
        let mut result = blst_p1::default();
        unsafe {
            blst_p1_double(&mut result, &self.0);
        }
        Self(result)
    }

    fn sub(&self, b: &Self) -> Self {
        let mut b_negative: FsG1 = *b;
        let mut ret = Self::default();
        unsafe {
            blst_p1_cneg(&mut b_negative.0, true);
            blst_p1_add_or_double(&mut ret.0, &self.0, &b_negative.0);
            ret
        }
    }

    fn equals(&self, b: &Self) -> bool {
        unsafe { blst_p1_is_equal(&self.0, &b.0) }
    }
}

impl Clone for FsG1 {
    fn clone(&self) -> Self {
        FsG1(self.0)
    }
}

impl Copy for FsG1 {}

impl G1Mul<FsFr> for FsG1 {
    fn mul(&self, b: &FsFr) -> Self {
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
}

extern crate alloc;

use core::ptr;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use blst::p1_affines;
use blst::{
    blst_fp, blst_p1, blst_p1_add, blst_p1_add_or_double, blst_p1_affine, blst_p1_cneg,
    blst_p1_compress, blst_p1_double, blst_p1_from_affine, blst_p1_in_g1, blst_p1_is_equal,
    blst_p1_is_inf, blst_p1_mult, blst_p1_uncompress, blst_scalar, blst_scalar_from_fr, BLST_ERROR,
};
use kzg::common_utils::log_2_byte;
use kzg::eip_4844::BYTES_PER_G1;
use kzg::msm::precompute::PrecomputationTable;
use kzg::G1Affine;
use kzg::G1GetFp;
use kzg::G1LinComb;
use kzg::G1ProjAddAffine;
use kzg::{G1Mul, G1};

use crate::consts::{G1_GENERATOR, G1_IDENTITY, G1_NEGATIVE_GENERATOR};
use crate::kzg_proofs::g1_linear_combination;
use crate::types::fr::FsFr;

use super::fp::FsFp;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct FsG1(pub blst_p1);

impl FsG1 {
    pub(crate) const fn from_xyz(x: blst_fp, y: blst_fp, z: blst_fp) -> Self {
        FsG1(blst_p1 { x, y, z })
    }
}

impl G1 for FsG1 {
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
        let result: FsG1 = G1_GENERATOR;
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
                let mut tmp = blst_p1_affine::default();
                let mut g1 = blst_p1::default();
                unsafe {
                    // The uncompress routine also checks that the point is on the curve
                    if blst_p1_uncompress(&mut tmp, bytes.as_ptr()) != BLST_ERROR::BLST_SUCCESS {
                        return Err("Failed to uncompress".to_string());
                    }
                    blst_p1_from_affine(&mut g1, &tmp);
                }
                Ok(FsG1(g1))
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

    fn add_or_dbl(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_p1_add_or_double(&mut ret.0, &self.0, &b.0);
        }
        ret
    }

    fn is_inf(&self) -> bool {
        unsafe { blst_p1_is_inf(&self.0) }
    }

    fn is_valid(&self) -> bool {
        unsafe {
            // The point must be on the right subgroup
            blst_p1_in_g1(&self.0)
        }
    }

    fn dbl(&self) -> Self {
        let mut result = blst_p1::default();
        unsafe {
            blst_p1_double(&mut result, &self.0);
        }
        Self(result)
    }

    fn add(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_p1_add(&mut ret.0, &self.0, &b.0);
        }
        ret
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

    const ZERO: Self = Self(blst_p1 {
        x: blst_fp {
            l: [
                8505329371266088957,
                17002214543764226050,
                6865905132761471162,
                8632934651105793861,
                6631298214892334189,
                1582556514881692819,
            ],
        },
        y: blst_fp {
            l: [
                8505329371266088957,
                17002214543764226050,
                6865905132761471162,
                8632934651105793861,
                6631298214892334189,
                1582556514881692819,
            ],
        },
        z: blst_fp {
            l: [0, 0, 0, 0, 0, 0],
        },
    });

    fn add_or_dbl_assign(&mut self, b: &Self) {
        unsafe {
            blst::blst_p1_add_or_double(&mut self.0, &self.0, &b.0);
        }
    }

    fn add_assign(&mut self, b: &Self) {
        unsafe {
            blst::blst_p1_add(&mut self.0, &self.0, &b.0);
        }
    }

    fn dbl_assign(&mut self) {
        unsafe {
            blst::blst_p1_double(&mut self.0, &self.0);
        }
    }
}

impl G1GetFp<FsFp> for FsG1 {
    fn x(&self) -> &FsFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&self.0.x)
        }
    }

    fn y(&self) -> &FsFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&self.0.y)
        }
    }

    fn z(&self) -> &FsFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&self.0.z)
        }
    }

    fn x_mut(&mut self) -> &mut FsFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&mut self.0.x)
        }
    }

    fn y_mut(&mut self) -> &mut FsFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&mut self.0.y)
        }
    }

    fn z_mut(&mut self) -> &mut FsFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&mut self.0.z)
        }
    }
}

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

impl G1LinComb<FsFr, FsFp, FsG1Affine> for FsG1 {
    fn g1_lincomb(
        points: &[Self],
        scalars: &[FsFr],
        len: usize,
        precomputation: Option<&PrecomputationTable<FsFr, Self, FsFp, FsG1Affine>>,
    ) -> Self {
        let mut out = FsG1::default();
        g1_linear_combination(&mut out, points, scalars, len, precomputation);
        out
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct FsG1Affine(pub blst_p1_affine);

impl G1Affine<FsG1, FsFp> for FsG1Affine {
    const ZERO: Self = Self(blst_p1_affine {
        x: {
            blst_fp {
                l: [0, 0, 0, 0, 0, 0],
            }
        },
        y: {
            blst_fp {
                l: [0, 0, 0, 0, 0, 0],
            }
        },
    });

    fn into_affine(g1: &FsG1) -> Self {
        let mut ret: Self = Default::default();
        unsafe {
            blst::blst_p1_to_affine(&mut ret.0, &g1.0);
        }
        ret
    }

    fn into_affines_loc(out: &mut [Self], g1: &[FsG1]) {
        let p: [*const blst_p1; 2] = [g1.as_ptr() as *const blst_p1, ptr::null()];
        unsafe {
            blst::blst_p1s_to_affine(out.as_mut_ptr() as *mut blst_p1_affine, &p[0], g1.len());
        }
    }

    fn into_affines(g1: &[FsG1]) -> Vec<Self> {
        let points =
            unsafe { core::slice::from_raw_parts(g1.as_ptr() as *const blst_p1, g1.len()) };
        let points = p1_affines::from(points);
        unsafe {
            // Transmute safe due to repr(C) on FsG1Affine
            core::mem::transmute(points)
        }
    }

    fn to_proj(&self) -> FsG1 {
        let mut ret: FsG1 = Default::default();
        unsafe {
            blst::blst_p1_from_affine(&mut ret.0, &self.0);
        }
        ret
    }

    fn x(&self) -> &FsFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&self.0.x)
        }
    }

    fn y(&self) -> &FsFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&self.0.y)
        }
    }

    fn is_infinity(&self) -> bool {
        unsafe { blst::blst_p1_affine_is_inf(&self.0) }
    }

    fn x_mut(&mut self) -> &mut FsFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&mut self.0.x)
        }
    }

    fn y_mut(&mut self) -> &mut FsFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&mut self.0.y)
        }
    }
}

pub struct FsG1ProjAddAffine;
impl G1ProjAddAffine<FsG1, FsFp, FsG1Affine> for FsG1ProjAddAffine {
    fn add_assign_affine(proj: &mut FsG1, aff: &FsG1Affine) {
        unsafe {
            blst::blst_p1_add_affine(&mut proj.0, &proj.0, &aff.0);
        }
    }

    fn add_or_double_assign_affine(proj: &mut FsG1, aff: &FsG1Affine) {
        unsafe {
            blst::blst_p1_add_or_double_affine(&mut proj.0, &proj.0, &aff.0);
        }
    }
}

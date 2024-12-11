extern crate alloc;

use core::ptr;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use blst::blst_p1;
use blst::blst_p1_affine;
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
use crate::mcl_methods::mcl_fp;
use crate::mcl_methods::mcl_g1;
use crate::types::fr::FsFr;

use super::fp::FsFp;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct FsG1(pub mcl_g1);

impl FsG1 {
    pub(crate) const fn from_xyz(x: mcl_fp, y: mcl_fp, z: mcl_fp) -> Self {
        FsG1(mcl_g1 { x, y, z })
    }

    pub const fn from_blst_p1(p1: blst_p1) -> Self {
        todo!();

        // Self(blst_p1_into_pc_g1projective(&p1))
    }

    pub const fn to_blst_p1(&self) -> blst_p1 {
        todo!()
        // pc_g1projective_into_blst_p1(self.0)
    }
}

impl G1 for FsG1 {
    fn zero() -> Self {
        todo!()
    }

    fn identity() -> Self {
        todo!()
    }

    fn generator() -> Self {
        todo!()
    }

    fn negative_generator() -> Self {
        todo!()
    }

    fn rand() -> Self {
        todo!()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        todo!()
    }

    fn from_hex(hex: &str) -> Result<Self, String> {
        todo!()
    }

    fn to_bytes(&self) -> [u8; 48] {
        todo!()
    }

    fn add_or_dbl(&self, b: &Self) -> Self {
        todo!()
    }

    fn is_inf(&self) -> bool {
        todo!()
    }

    fn is_valid(&self) -> bool {
        todo!()
    }

    fn dbl(&self) -> Self {
        todo!()
    }

    fn add(&self, b: &Self) -> Self {
        todo!()
    }

    fn sub(&self, b: &Self) -> Self {
        todo!()
    }

    fn equals(&self, b: &Self) -> bool {
        todo!()
    }

    fn add_or_dbl_assign(&mut self, b: &Self) {
        todo!()
    }

    fn add_assign(&mut self, b: &Self) {
        todo!()
    }

    fn dbl_assign(&mut self) {
        todo!()
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
        todo!()
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
    fn zero() -> Self {
        todo!()
    }

    fn into_affine(g1: &FsG1) -> Self {
        todo!()
    }

    fn into_affines_loc(out: &mut [Self], g1: &[FsG1]) {
        todo!()
    }

    fn to_proj(&self) -> FsG1 {
        todo!()
    }

    fn x(&self) -> &FsFp {
        todo!()
    }

    fn y(&self) -> &FsFp {
        todo!()
    }

    fn x_mut(&mut self) -> &mut FsFp {
        todo!()
    }

    fn y_mut(&mut self) -> &mut FsFp {
        todo!()
    }

    fn is_infinity(&self) -> bool {
        todo!()
    }
}

pub struct FsG1ProjAddAffine;
impl G1ProjAddAffine<FsG1, FsFp, FsG1Affine> for FsG1ProjAddAffine {
    fn add_assign_affine(proj: &mut FsG1, aff: &FsG1Affine) {
        todo!()
    }

    fn add_or_double_assign_affine(proj: &mut FsG1, aff: &FsG1Affine) {
        todo!()
    }
}

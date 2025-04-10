extern crate alloc;

use core::hash::Hash;
use core::ops::Add;
use core::ops::Sub;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;

use blst::blst_fp;
use blst::blst_p1;
use blst::blst_p1_affine;
use blst::blst_p1_in_g1;
use kzg::eip_4844::BYTES_PER_G1;
use kzg::msm::precompute::PrecomputationTable;
use kzg::G1Affine;
use kzg::G1GetFp;
use kzg::G1LinComb;
use kzg::G1ProjAddAffine;
use kzg::{G1Mul, G1};

use crate::consts::{G1_GENERATOR, G1_IDENTITY, G1_NEGATIVE_GENERATOR};
use crate::kzg_proofs::g1_linear_combination;
use crate::mcl_methods::mclBnFp_neg;
use crate::mcl_methods::mcl_fp;
use crate::mcl_methods::mcl_g1;
use crate::mcl_methods::try_init_mcl;
use crate::types::fr::MclFr;

use super::fp::MclFp;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct MclG1(pub mcl_g1);

impl Hash for MclG1 {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.x.d.hash(state);
        self.0.y.d.hash(state);
        self.0.z.d.hash(state);
    }
}

impl MclG1 {
    pub(crate) const fn from_xyz(x: mcl_fp, y: mcl_fp, z: mcl_fp) -> Self {
        MclG1(mcl_g1 { x, y, z })
    }

    pub fn from_blst_p1(p1: blst_p1) -> Self {
        Self(mcl_g1 {
            x: mcl_fp { d: p1.x.l },
            y: mcl_fp { d: p1.y.l },
            z: mcl_fp { d: p1.z.l },
        })

        // Self(blst_p1_into_pc_g1projective(&p1))
    }

    pub const fn to_blst_p1(&self) -> blst_p1 {
        blst_p1 {
            x: blst_fp { l: self.0.x.d },
            y: blst_fp { l: self.0.y.d },
            z: blst_fp { l: self.0.z.d },
        }
        // pc_g1projective_into_blst_p1(self.0)
    }
}

impl G1 for MclG1 {
    fn zero() -> Self {
        try_init_mcl();

        Self(mcl_g1 {
            x: mcl_fp {
                d: [
                    8505329371266088957,
                    17002214543764226050,
                    6865905132761471162,
                    8632934651105793861,
                    6631298214892334189,
                    1582556514881692819,
                ],
            },
            y: mcl_fp {
                d: [
                    8505329371266088957,
                    17002214543764226050,
                    6865905132761471162,
                    8632934651105793861,
                    6631298214892334189,
                    1582556514881692819,
                ],
            },
            z: mcl_fp {
                d: [0, 0, 0, 0, 0, 0],
            },
        })
    }

    fn identity() -> Self {
        try_init_mcl();

        G1_IDENTITY
    }

    fn generator() -> Self {
        try_init_mcl();

        G1_GENERATOR
    }

    fn negative_generator() -> Self {
        try_init_mcl();

        G1_NEGATIVE_GENERATOR
    }

    #[cfg(feature = "rand")]
    fn rand() -> Self {
        try_init_mcl();

        let result: MclG1 = G1_GENERATOR;
        result.mul(&kzg::Fr::rand())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        try_init_mcl();

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
                    if blst::blst_p1_uncompress(&mut tmp, bytes.as_ptr())
                        != blst::BLST_ERROR::BLST_SUCCESS
                    {
                        return Err("Failed to uncompress".to_string());
                    }
                    blst::blst_p1_from_affine(&mut g1, &tmp);
                }
                Ok(MclG1::from_blst_p1(g1))
            })
    }

    fn from_hex(hex: &str) -> Result<Self, String> {
        let bytes = hex::decode(&hex[2..]).unwrap();
        Self::from_bytes(&bytes)
    }

    fn to_bytes(&self) -> [u8; 48] {
        try_init_mcl();

        let mut out = [0u8; BYTES_PER_G1];
        unsafe {
            blst::blst_p1_compress(out.as_mut_ptr(), &self.to_blst_p1());
        }
        out
    }

    fn add_or_dbl(&self, b: &Self) -> Self {
        try_init_mcl();

        let mut out = mcl_g1::default();
        mcl_g1::add(&mut out, &self.0, &b.0);
        Self(out)
    }

    fn is_inf(&self) -> bool {
        try_init_mcl();

        self.0.get_str(0).eq("0")
    }

    fn is_valid(&self) -> bool {
        try_init_mcl();

        let blst = self.to_blst_p1();

        unsafe { blst_p1_in_g1(&blst) }
    }

    fn dbl(&self) -> Self {
        try_init_mcl();

        let mut out = mcl_g1::default();
        mcl_g1::dbl(&mut out, &self.0);
        Self(out)
    }

    fn add(&self, b: &Self) -> Self {
        try_init_mcl();

        Self(self.0.add(&b.0))
    }

    fn sub(&self, b: &Self) -> Self {
        try_init_mcl();

        Self(self.0.sub(&b.0))
    }

    fn equals(&self, b: &Self) -> bool {
        try_init_mcl();

        mcl_g1::eq(&self.0, &b.0)
    }

    fn add_or_dbl_assign(&mut self, b: &Self) {
        try_init_mcl();

        self.0 = self.0.add(&b.0);
    }

    fn add_assign(&mut self, b: &Self) {
        try_init_mcl();

        self.0 = self.0.add(&b.0);
    }

    fn dbl_assign(&mut self) {
        try_init_mcl();

        let mut r = mcl_g1::default();
        mcl_g1::dbl(&mut r, &self.0);
        self.0 = r;
    }
}

impl G1GetFp<MclFp> for MclG1 {
    fn x(&self) -> &MclFp {
        try_init_mcl();

        unsafe {
            // Transmute safe due to repr(C) on MclFp
            core::mem::transmute(&self.0.x)
        }
    }

    fn y(&self) -> &MclFp {
        try_init_mcl();

        unsafe {
            // Transmute safe due to repr(C) on MclFp
            core::mem::transmute(&self.0.y)
        }
    }

    fn z(&self) -> &MclFp {
        try_init_mcl();

        unsafe {
            // Transmute safe due to repr(C) on MclFp
            core::mem::transmute(&self.0.z)
        }
    }

    fn x_mut(&mut self) -> &mut MclFp {
        try_init_mcl();

        unsafe {
            // Transmute safe due to repr(C) on MclFp
            core::mem::transmute(&mut self.0.x)
        }
    }

    fn y_mut(&mut self) -> &mut MclFp {
        try_init_mcl();

        unsafe {
            // Transmute safe due to repr(C) on MclFp
            core::mem::transmute(&mut self.0.y)
        }
    }

    fn z_mut(&mut self) -> &mut MclFp {
        try_init_mcl();

        unsafe {
            // Transmute safe due to repr(C) on MclFp
            core::mem::transmute(&mut self.0.z)
        }
    }
}

impl G1Mul<MclFr> for MclG1 {
    fn mul(&self, b: &MclFr) -> Self {
        try_init_mcl();

        let mut out = MclG1::default();
        mcl_g1::mul(&mut out.0, &self.0, &b.0);
        out
    }
}

impl G1LinComb<MclFr, MclFp, MclG1Affine, MclG1ProjAddAffine> for MclG1 {
    fn g1_lincomb(
        points: &[Self],
        scalars: &[MclFr],
        len: usize,
        precomputation: Option<
            &PrecomputationTable<MclFr, Self, MclFp, MclG1Affine, MclG1ProjAddAffine>,
        >,
    ) -> Self {
        try_init_mcl();

        let mut out = MclG1::default();
        g1_linear_combination(&mut out, points, scalars, len, precomputation);
        out
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct MclG1Affine {
    pub x: mcl_fp,
    pub y: mcl_fp,
}

impl G1Affine<MclG1, MclFp> for MclG1Affine {
    fn zero() -> Self {
        try_init_mcl();

        Self {
            x: {
                mcl_fp {
                    d: [0, 0, 0, 0, 0, 0],
                }
            },
            y: {
                mcl_fp {
                    d: [0, 0, 0, 0, 0, 0],
                }
            },
        }
    }

    fn into_affine(g1: &MclG1) -> Self {
        try_init_mcl();

        let mut out: mcl_g1 = Default::default();
        mcl_g1::normalize(&mut out, &g1.0);

        Self { x: out.x, y: out.y }
    }

    fn into_affines_loc(out: &mut [Self], g1: &[MclG1]) {
        try_init_mcl();

        for (i, g) in g1.iter().enumerate() {
            out[i] = Self::into_affine(g);
        }
    }

    fn to_proj(&self) -> MclG1 {
        try_init_mcl();

        let mut ret: MclG1 = MclG1::generator();

        ret.0.x = self.x;
        ret.0.y = self.y;

        ret
    }

    fn x(&self) -> &MclFp {
        try_init_mcl();

        unsafe {
            // Transmute safe due to repr(C) on MclFp
            core::mem::transmute(&self.x)
        }
    }

    fn y(&self) -> &MclFp {
        try_init_mcl();

        unsafe {
            // Transmute safe due to repr(C) on MclFp
            core::mem::transmute(&self.y)
        }
    }

    fn x_mut(&mut self) -> &mut MclFp {
        try_init_mcl();

        unsafe {
            // Transmute safe due to repr(C) on MclFp
            core::mem::transmute(&mut self.x)
        }
    }

    fn y_mut(&mut self) -> &mut MclFp {
        try_init_mcl();

        unsafe {
            // Transmute safe due to repr(C) on MclFp
            core::mem::transmute(&mut self.y)
        }
    }

    fn is_infinity(&self) -> bool {
        todo!()
    }

    fn neg(&self) -> Self {
        try_init_mcl();

        let mut output = *self;

        unsafe {
            mclBnFp_neg(&mut output.y, &output.x);
        }
        output
    }

    fn from_xy(x: MclFp, y: MclFp) -> Self {
        Self { x: x.0, y: y.0 }
    }
}

#[derive(Debug)]
pub struct MclG1ProjAddAffine;

impl G1ProjAddAffine<MclG1, MclFp, MclG1Affine> for MclG1ProjAddAffine {
    fn add_assign_affine(_proj: &mut MclG1, _aff: &MclG1Affine) {
        todo!()
    }

    fn add_or_double_assign_affine(_proj: &mut MclG1, _aff: &MclG1Affine) {
        todo!()
    }
}

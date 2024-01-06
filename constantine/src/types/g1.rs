extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use constantine::ctt_codec_ecc_status;
use kzg::msm::precompute::PrecomputationTable;
use kzg::G1LinComb;

use core::fmt::{Debug, Formatter};

use crate::kzg_proofs::g1_linear_combination;
use crate::types::fp::CtFp;
use crate::types::fr::CtFr;
use crate::utils::ptr_transmute;
use crate::utils::ptr_transmute_mut;
use kzg::common_utils::log_2_byte;
use kzg::eip_4844::BYTES_PER_G1;
use kzg::G1Affine;
use kzg::G1GetFp;
use kzg::G1ProjAddAffine;
use kzg::{G1Mul, G1};

use crate::consts::{G1_GENERATOR, G1_IDENTITY, G1_NEGATIVE_GENERATOR};
// use crate::kzg_proofs::g1_linear_combination;

use constantine_sys as constantine;

use constantine_sys::{
    bls12_381_fp, bls12_381_g1_aff, bls12_381_g1_jac, ctt_bls12_381_g1_jac_from_affine,
};

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CtG1(pub bls12_381_g1_jac);

impl PartialEq for CtG1 {
    fn eq(&self, other: &Self) -> bool {
        unsafe { constantine::ctt_bls12_381_g1_jac_is_eq(&self.0, &other.0) != 0 }
    }
}

impl Debug for CtG1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "CtG1({:?}, {:?}, {:?})",
            self.0.x.limbs, self.0.y.limbs, self.0.z.limbs
        )
    }
}

impl CtG1 {
    pub(crate) const fn from_xyz(x: bls12_381_fp, y: bls12_381_fp, z: bls12_381_fp) -> Self {
        CtG1(bls12_381_g1_jac { x, y, z })
    }

    pub const fn from_blst_p1(p1: blst::blst_p1) -> Self {
        Self(bls12_381_g1_jac {
            x: bls12_381_fp { limbs: p1.x.l },
            y: bls12_381_fp { limbs: p1.y.l },
            z: bls12_381_fp { limbs: p1.z.l },
        })
    }

    pub const fn to_blst_p1(&self) -> blst::blst_p1 {
        blst::blst_p1 {
            x: blst::blst_fp { l: self.0.x.limbs },
            y: blst::blst_fp { l: self.0.y.limbs },
            z: blst::blst_fp { l: self.0.z.limbs },
        }
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
                    let res = constantine::ctt_bls12_381_deserialize_g1_compressed(
                        &mut tmp,
                        bytes.as_ptr(),
                    );
                    if res != ctt_codec_ecc_status::cttCodecEcc_Success
                        && res != ctt_codec_ecc_status::cttCodecEcc_PointAtInfinity
                    {
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
            let _ = constantine::ctt_bls12_381_serialize_g1_compressed(
                out.as_mut_ptr(),
                &CtG1Affine::into_affine(self).0,
            );
        }
        out
    }

    fn add_or_dbl(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            constantine::ctt_bls12_381_g1_jac_sum(&mut ret.0, &self.0, &b.0);
        }
        ret
    }

    fn is_inf(&self) -> bool {
        unsafe { constantine::ctt_bls12_381_g1_jac_is_inf(&self.0) != 0 }
    }

    fn is_valid(&self) -> bool {
        unsafe {
            constantine::ctt_bls12_381_validate_g1(&CtG1Affine::into_affine(self).0)
                == ctt_codec_ecc_status::cttCodecEcc_Success
        }
    }

    fn dbl(&self) -> Self {
        let mut result = bls12_381_g1_jac::default();
        unsafe {
            constantine::ctt_bls12_381_g1_jac_double(&mut result, &self.0);
        }
        Self(result)
    }

    fn add(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            constantine::ctt_bls12_381_g1_jac_sum(&mut ret.0, &self.0, &b.0);
        }
        ret
    }

    fn sub(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            constantine::ctt_bls12_381_g1_jac_diff(&mut ret.0, &self.0, &b.0);
        }
        ret
    }

    fn equals(&self, b: &Self) -> bool {
        unsafe { constantine::ctt_bls12_381_g1_jac_is_eq(&self.0, &b.0) != 0 }
    }

    const ZERO: Self = CtG1::from_xyz(
        bls12_381_fp {
            limbs: [
                8505329371266088957,
                17002214543764226050,
                6865905132761471162,
                8632934651105793861,
                6631298214892334189,
                1582556514881692819,
            ],
        },
        bls12_381_fp {
            limbs: [
                8505329371266088957,
                17002214543764226050,
                6865905132761471162,
                8632934651105793861,
                6631298214892334189,
                1582556514881692819,
            ],
        },
        bls12_381_fp { limbs: [0; 6] },
    );

    fn add_or_dbl_assign(&mut self, b: &Self) {
        unsafe {
            constantine::ctt_bls12_381_g1_jac_add_in_place(&mut self.0, &b.0);
        }
    }

    fn add_assign(&mut self, b: &Self) {
        unsafe {
            constantine::ctt_bls12_381_g1_jac_add_in_place(&mut self.0, &b.0);
        }
    }

    fn dbl_assign(&mut self) {
        unsafe {
            constantine::ctt_bls12_381_g1_jac_double_in_place(&mut self.0);
        }
    }
}

impl G1Mul<CtFr> for CtG1 {
    fn mul(&self, b: &CtFr) -> Self {
        // FIXME: No transmute here, use constantine
        let mut scalar = blst::blst_scalar::default();
        unsafe {
            blst::blst_scalar_from_fr(&mut scalar, ptr_transmute(&b.0));
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
                blst::blst_p1_mult(
                    ptr_transmute_mut(&mut result.0),
                    ptr_transmute(&self.0),
                    &(scalar.b[0]),
                    8 * i - 7 + log_2_byte(scalar.b[i - 1]),
                );
            }
        }
        result
    }
}

impl G1LinComb<CtFr, CtFp, CtG1Affine> for CtG1 {
    fn g1_lincomb(
        points: &[Self],
        scalars: &[CtFr],
        len: usize,
        precomputation: Option<&PrecomputationTable<CtFr, Self, CtFp, CtG1Affine>>,
    ) -> Self {
        let mut out = CtG1::default();
        g1_linear_combination(&mut out, points, scalars, len, precomputation);
        out
    }
}

impl G1GetFp<CtFp> for CtG1 {
    fn x(&self) -> &CtFp {
        unsafe {
            // Transmute safe due to repr(C) on CtFp
            core::mem::transmute(&self.0.x)
        }
    }

    fn y(&self) -> &CtFp {
        unsafe {
            // Transmute safe due to repr(C) on CtFp
            core::mem::transmute(&self.0.y)
        }
    }

    fn z(&self) -> &CtFp {
        unsafe {
            // Transmute safe due to repr(C) on CtFp
            core::mem::transmute(&self.0.z)
        }
    }

    fn x_mut(&mut self) -> &mut CtFp {
        unsafe {
            // Transmute safe due to repr(C) on CtFp
            core::mem::transmute(&mut self.0.x)
        }
    }

    fn y_mut(&mut self) -> &mut CtFp {
        unsafe {
            // Transmute safe due to repr(C) on CtFp
            core::mem::transmute(&mut self.0.y)
        }
    }

    fn z_mut(&mut self) -> &mut CtFp {
        unsafe {
            // Transmute safe due to repr(C) on CtFp
            core::mem::transmute(&mut self.0.z)
        }
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct CtG1Affine(pub constantine::bls12_381_g1_aff);

impl PartialEq for CtG1Affine {
    fn eq(&self, other: &Self) -> bool {
        unsafe { constantine::ctt_bls12_381_g1_aff_is_eq(&self.0, &other.0) != 0 }
    }
}

impl Debug for CtG1Affine {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "CtG1Affine({:?}, {:?})", self.0.x.limbs, self.0.y.limbs)
    }
}

impl G1Affine<CtG1, CtFp> for CtG1Affine {
    const ZERO: Self = Self(bls12_381_g1_aff {
        x: {
            bls12_381_fp {
                limbs: [0, 0, 0, 0, 0, 0],
            }
        },
        y: {
            bls12_381_fp {
                limbs: [0, 0, 0, 0, 0, 0],
            }
        },
    });

    fn into_affine(g1: &CtG1) -> Self {
        let mut ret: Self = Default::default();
        unsafe {
            constantine::ctt_bls12_381_g1_jac_affine(&mut ret.0, &g1.0);
        }
        ret
    }

    fn into_affines_loc(out: &mut [Self], g1: &[CtG1]) {
        unsafe{
            constantine::ctt_bls12_381_g1_jac_batch_affine(core::mem::transmute(out.as_mut_ptr()), core::mem::transmute(g1.as_ptr()), g1.len());
        }
    }

    fn to_proj(&self) -> CtG1 {
        let mut ret: CtG1 = Default::default();
        unsafe {
            constantine::ctt_bls12_381_g1_jac_from_affine(&mut ret.0, &self.0);
        }
        ret
    }

    fn x(&self) -> &CtFp {
        unsafe {
            // Transmute safe due to repr(C) on CtFp
            core::mem::transmute(&self.0.x)
        }
    }

    fn y(&self) -> &CtFp {
        unsafe {
            // Transmute safe due to repr(C) on CtFp
            core::mem::transmute(&self.0.y)
        }
    }

    fn is_infinity(&self) -> bool {
        unsafe { constantine::ctt_bls12_381_g1_aff_is_inf(&self.0) != 0 }
    }

    fn x_mut(&mut self) -> &mut CtFp {
        unsafe {
            // Transmute safe due to repr(C) on CtFp
            core::mem::transmute(&mut self.0.x)
        }
    }

    fn y_mut(&mut self) -> &mut CtFp {
        unsafe {
            // Transmute safe due to repr(C) on CtFp
            core::mem::transmute(&mut self.0.y)
        }
    }
}

pub struct CtG1ProjAddAffine;
impl G1ProjAddAffine<CtG1, CtFp, CtG1Affine> for CtG1ProjAddAffine {
    fn add_assign_affine(proj: &mut CtG1, aff: &CtG1Affine) {
        let mut g1_jac = bls12_381_g1_jac::default();
        unsafe {
            constantine::ctt_bls12_381_g1_jac_from_affine(&mut g1_jac, &aff.0);
            constantine::ctt_bls12_381_g1_jac_add_in_place(&mut proj.0, &g1_jac);
        }
    }

    fn add_or_double_assign_affine(proj: &mut CtG1, aff: &CtG1Affine) {
        let mut g1_jac = bls12_381_g1_jac::default();
        unsafe {
            constantine::ctt_bls12_381_g1_jac_from_affine(&mut g1_jac, &aff.0);
            constantine::ctt_bls12_381_g1_jac_add_in_place(&mut proj.0, &g1_jac);
        }
    }
}

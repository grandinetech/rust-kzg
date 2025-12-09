extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use arbitrary::Arbitrary;
use constantine::ctt_codec_ecc_status;
use kzg::eth::c_bindings::blst_fp;
use kzg::eth::c_bindings::blst_p1;
use kzg::msm::precompute::PrecomputationTable;
use kzg::G1LinComb;

use core::{
    fmt::{Debug, Formatter},
    hash::Hash,
};

use crate::kzg_proofs::g1_linear_combination;
use crate::types::fp::CtFp;
use crate::types::fr::CtFr;

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

impl Hash for CtG1 {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.x.limbs.hash(state);
        self.0.y.limbs.hash(state);
        self.0.z.limbs.hash(state);
    }
}

impl PartialEq for CtG1 {
    fn eq(&self, other: &Self) -> bool {
        unsafe { constantine::ctt_bls12_381_g1_jac_is_eq(&self.0, &other.0) != 0 }
    }
}

impl Eq for CtG1 {}

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

    pub const fn from_blst_p1(p1: blst_p1) -> Self {
        unsafe {
            Self(bls12_381_g1_jac {
                x: bls12_381_fp {
                    limbs: core::mem::transmute::<[u64; 6], [usize; 6]>(p1.x.l),
                },
                y: bls12_381_fp {
                    limbs: core::mem::transmute::<[u64; 6], [usize; 6]>(p1.y.l),
                },
                z: bls12_381_fp {
                    limbs: core::mem::transmute::<[u64; 6], [usize; 6]>(p1.z.l),
                },
            })
        }
    }

    pub const fn to_blst_p1(&self) -> blst_p1 {
        unsafe {
            blst_p1 {
                x: blst_fp {
                    l: core::mem::transmute::<[usize; 6], [u64; 6]>(self.0.x.limbs),
                },
                y: blst_fp {
                    l: core::mem::transmute::<[usize; 6], [u64; 6]>(self.0.y.limbs),
                },
                z: blst_fp {
                    l: core::mem::transmute::<[usize; 6], [u64; 6]>(self.0.z.limbs),
                },
            }
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
        unsafe { constantine::ctt_bls12_381_g1_jac_is_neutral(&self.0) != 0 }
    }

    fn is_valid(&self) -> bool {
        unsafe {
            matches!(
                constantine::ctt_bls12_381_validate_g1(&CtG1Affine::into_affine(self).0),
                ctt_codec_ecc_status::cttCodecEcc_Success
                    | ctt_codec_ecc_status::cttCodecEcc_PointAtInfinity
            )
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

    fn zero() -> Self {
        CtG1::from_xyz(
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
        )
    }

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
        let mut result = *self;
        unsafe {
            constantine::ctt_bls12_381_g1_jac_scalar_mul_fr_coef(&mut result.0, &b.0);
        }
        result
    }
}

impl G1LinComb<CtFr, CtFp, CtG1Affine, CtG1ProjAddAffine> for CtG1 {
    fn g1_lincomb(
        points: &[Self],
        scalars: &[CtFr],
        len: usize,
        precomputation: Option<
            &PrecomputationTable<CtFr, Self, CtFp, CtG1Affine, CtG1ProjAddAffine>,
        >,
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

impl<'a> Arbitrary<'a> for CtG1Affine {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(CtG1Affine::into_affine(
            &CtG1::generator().mul(&u.arbitrary()?),
        ))
    }
}

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
    fn zero() -> Self {
        Self(bls12_381_g1_aff {
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
        })
    }

    fn into_affine(g1: &CtG1) -> Self {
        let mut ret: Self = Default::default();
        unsafe {
            constantine::ctt_bls12_381_g1_jac_affine(&mut ret.0, &g1.0);
        }
        ret
    }

    fn into_affines_loc(out: &mut [Self], g1: &[CtG1]) {
        if g1.is_empty() {
            return;
        }

        unsafe {
            constantine::ctt_bls12_381_g1_jac_batch_affine(
                core::mem::transmute::<*mut CtG1Affine, *const constantine_sys::bls12_381_g1_aff>(
                    out.as_mut_ptr(),
                ),
                core::mem::transmute::<*const CtG1, *const constantine_sys::bls12_381_g1_jac>(
                    g1.as_ptr(),
                ),
                g1.len(),
            );
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
        unsafe { constantine::ctt_bls12_381_g1_aff_is_neutral(&self.0) != 0 }
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

    fn neg(&self) -> Self {
        let mut output = *self;
        unsafe {
            constantine::ctt_bls12_381_g1_aff_neg_in_place(&mut output.0);
        }
        output
    }

    fn from_xy(x: CtFp, y: CtFp) -> Self {
        Self(bls12_381_g1_aff { x: x.0, y: y.0 })
    }

    fn to_bytes_uncompressed(&self) -> [u8; 96] {
        let mut out = [0u8; 96];
        // Serialize: 48 bytes x (big-endian) || 48 bytes y (big-endian)
        // limbs are stored in little-endian, so limbs[5] is most significant
        for i in 0..6 {
            let bytes = self.0.x.limbs[5 - i].to_be_bytes();
            out[i * 8..(i + 1) * 8].copy_from_slice(&bytes);
        }
        for i in 0..6 {
            let bytes = self.0.y.limbs[5 - i].to_be_bytes();
            out[48 + i * 8..48 + (i + 1) * 8].copy_from_slice(&bytes);
        }
        out
    }

    fn from_bytes_uncompressed(bytes: [u8; 96]) -> Result<Self, String> {
        let mut x_limbs: [usize; 6] = [0; 6];
        let mut y_limbs: [usize; 6] = [0; 6];
        
        // Deserialize: bytes come in big-endian
        // We need to store them in little-endian limbs array
        for i in 0..6 {
            let mut limb_bytes = [0u8; 8];
            limb_bytes.copy_from_slice(&bytes[i * 8..(i + 1) * 8]);
            x_limbs[5 - i] = usize::from_be_bytes(limb_bytes);
        }
        for i in 0..6 {
            let mut limb_bytes = [0u8; 8];
            limb_bytes.copy_from_slice(&bytes[48 + i * 8..48 + (i + 1) * 8]);
            y_limbs[5 - i] = usize::from_be_bytes(limb_bytes);
        }
        
        let tmp = bls12_381_g1_aff {
            x: bls12_381_fp { limbs: x_limbs },
            y: bls12_381_fp { limbs: y_limbs },
        };
        
        // Validate point is on curve
        unsafe {
            match constantine::ctt_bls12_381_validate_g1(&tmp) {
                ctt_codec_ecc_status::cttCodecEcc_Success 
                | ctt_codec_ecc_status::cttCodecEcc_PointAtInfinity => Ok(CtG1Affine(tmp)),
                _ => Err("Point is not on the curve".to_string()),
            }
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

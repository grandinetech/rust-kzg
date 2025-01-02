use crate::consts::SCALE2_ROOT_OF_UNITY;
use crate::fft_g1::{fft_g1_fast, g1_linear_combination};
pub use crate::kzg_proofs::{expand_root_of_unity, pairings_verify, LFFTSettings, LKZGSettings};
// use crate::poly::{poly_fast_div, poly_inverse, poly_long_div, poly_mul_direct, poly_mul_fft};
use crate::recover::{scale_poly, unscale_poly};
use crate::utils::{
    blst_fp_into_pc_fq, blst_p1_into_pc_g1projective, blst_p2_into_pc_g2projective,
    pc_g1projective_into_blst_p1, pc_g2projective_into_blst_p2, PolyData,
};
use crate::P2;
use ark_bls12_381::{g1, g2, G1Affine};
use ark_ec::{models::short_weierstrass_jacobian::GroupProjective, ProjectiveCurve};
use ark_ec::{AffineCurve, ModelParameters};
use ark_ff::{BigInteger, Field};
use ark_std::{One, Zero};
use blst::{
    blst_bendian_from_scalar, blst_fp, blst_fp2, blst_fr, blst_fr_add, blst_fr_cneg,
    blst_fr_eucl_inverse, blst_fr_from_scalar, blst_fr_from_uint64, blst_fr_inverse, blst_fr_mul,
    blst_fr_sqr, blst_fr_sub, blst_p1, blst_p1_affine, blst_p1_compress, blst_p1_from_affine,
    blst_p1_in_g1, blst_p1_uncompress, blst_p2, blst_p2_affine, blst_p2_from_affine,
    blst_p2_uncompress, blst_scalar, blst_scalar_fr_check, blst_scalar_from_bendian,
    blst_scalar_from_fr, blst_uint64_from_fr, BLST_ERROR,
};
use kzg::common_utils::{log2_u64, reverse_bit_order};
use kzg::eip_4844::{BYTES_PER_FIELD_ELEMENT, BYTES_PER_G1, BYTES_PER_G2};
use kzg::msm::precompute::{precompute, PrecomputationTable};
use kzg::{
    FFTFr, FFTSettings, FFTSettingsPoly, Fr as KzgFr, G1Affine as G1AffineTrait, G1Fp, G1GetFp,
    G1LinComb, G1Mul, G1ProjAddAffine, G2Mul, KZGSettings, PairingVerify, Poly, Scalar256, G1, G2,
};
use std::ops::{AddAssign, Sub};

extern crate alloc;
use alloc::sync::Arc;

// fn bytes_be_to_uint64(inp: &[u8]) -> u64 {
//     u64::from_be_bytes(inp.try_into().expect("Input wasn't 8 elements..."))
// }

// const BLS12_381_MOD_256: [u64; 4] = [
//     0xffffffff00000001,
//     0x53bda402fffe5bfe,
//     0x3339d80809a1d805,
//     0x73eda753299d7d48,
// ];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct ArkFr(pub blst_fr);

// impl Fr for ArkFr {

// fn bigint_check_mod_256(a: &[u64; 4]) -> bool {
//     let (_, overflow) = a[0].overflowing_sub(BLS12_381_MOD_256[0]);
//     let (_, overflow) = a[1].overflowing_sub(BLS12_381_MOD_256[1] + overflow as u64);
//     let (_, overflow) = a[2].overflowing_sub(BLS12_381_MOD_256[2] + overflow as u64);
//     let (_, overflow) = a[3].overflowing_sub(BLS12_381_MOD_256[3] + overflow as u64);
//     overflow
// }

impl KzgFr for ArkFr {
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
                let mut fr = blst_fr::default();
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
                let mut fr = blst_fr::default();
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
            blst_fr_sqr(&mut ret.0, &self.0);
        }

        ret
    }

    fn mul(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_mul(&mut ret.0, &self.0, &b.0);
        }

        ret
    }

    fn add(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_add(&mut ret.0, &self.0, &b.0);
        }

        ret
    }

    fn sub(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_sub(&mut ret.0, &self.0, &b.0);
        }

        ret
    }

    fn eucl_inverse(&self) -> Self {
        // Inverse and eucl inverse work the same way
        let mut ret = Self::default();
        unsafe {
            blst_fr_eucl_inverse(&mut ret.0, &self.0);
        }

        ret
    }

    fn negate(&self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_cneg(&mut ret.0, &self.0, true);
        }

        ret
    }

    fn inverse(&self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_inverse(&mut ret.0, &self.0);
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

    fn to_scalar(&self) -> Scalar256 {
        let mut blst_scalar = blst_scalar::default();
        unsafe {
            blst_scalar_from_fr(&mut blst_scalar, &self.0);
        }
        Scalar256::from_u8(&blst_scalar.b)
    }
}

#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct ArkG1(pub GroupProjective<g1::Parameters>);

impl ArkG1 {
    pub fn from_blst_p1(p1: blst_p1) -> Self {
        Self(blst_p1_into_pc_g1projective(&p1))
    }

    pub fn to_blst_p1(&self) -> blst_p1 {
        pc_g1projective_into_blst_p1(self.0)
    }
}

impl From<blst_p1> for ArkG1 {
    fn from(p1: blst_p1) -> Self {
        Self(blst_p1_into_pc_g1projective(&p1))
    }
}

impl G1 for ArkG1 {
    fn identity() -> Self {
        ArkG1::from_blst_p1(blst_p1 {
            x: blst_fp {
                l: [0, 0, 0, 0, 0, 0],
            },
            y: blst_fp {
                l: [0, 0, 0, 0, 0, 0],
            },
            z: blst_fp {
                l: [0, 0, 0, 0, 0, 0],
            },
        })
    }

    fn generator() -> Self {
        ArkG1::from_blst_p1(blst_p1 {
            x: blst_fp {
                l: [
                    0x5cb38790fd530c16,
                    0x7817fc679976fff5,
                    0x154f95c7143ba1c1,
                    0xf0ae6acdf3d0e747,
                    0xedce6ecc21dbf440,
                    0x120177419e0bfb75,
                ],
            },
            y: blst_fp {
                l: [
                    0xbaac93d50ce72271,
                    0x8c22631a7918fd8e,
                    0xdd595f13570725ce,
                    0x51ac582950405194,
                    0x0e1c8c3fad0059c0,
                    0x0bbc3efc5008a26a,
                ],
            },
            z: blst_fp {
                l: [
                    0x760900000002fffd,
                    0xebf4000bc40c0002,
                    0x5f48985753c758ba,
                    0x77ce585370525745,
                    0x5c071a97a256ec6d,
                    0x15f65ec3fa80e493,
                ],
            },
        })
    }

    fn negative_generator() -> Self {
        ArkG1::from_blst_p1(blst_p1 {
            x: blst_fp {
                l: [
                    0x5cb38790fd530c16,
                    0x7817fc679976fff5,
                    0x154f95c7143ba1c1,
                    0xf0ae6acdf3d0e747,
                    0xedce6ecc21dbf440,
                    0x120177419e0bfb75,
                ],
            },
            y: blst_fp {
                l: [
                    0xff526c2af318883a,
                    0x92899ce4383b0270,
                    0x89d7738d9fa9d055,
                    0x12caf35ba344c12a,
                    0x3cff1b76964b5317,
                    0x0e44d2ede9774430,
                ],
            },
            z: blst_fp {
                l: [
                    0x760900000002fffd,
                    0xebf4000bc40c0002,
                    0x5f48985753c758ba,
                    0x77ce585370525745,
                    0x5c071a97a256ec6d,
                    0x15f65ec3fa80e493,
                ],
            },
        })
    }

    #[cfg(feature = "rand")]
    fn rand() -> Self {
        use ark_ff::UniformRand;

        let mut rng = rand::thread_rng();
        Self(GroupProjective::rand(&mut rng))
    }

    #[allow(clippy::bind_instead_of_map)]
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
                let mut blst_affine = blst_p1_affine::default();
                let result = unsafe { blst_p1_uncompress(&mut blst_affine, bytes.as_ptr()) };

                if result != BLST_ERROR::BLST_SUCCESS {
                    return Err("Failed to deserialize G1".to_owned());
                }

                let mut blst_point = blst_p1::default();
                unsafe { blst_p1_from_affine(&mut blst_point, &blst_affine) };

                Ok(ArkG1::from_blst_p1(blst_point))
            })
    }

    fn from_hex(hex: &str) -> Result<Self, String> {
        let bytes = hex::decode(&hex[2..]).unwrap();
        Self::from_bytes(&bytes)
    }

    fn to_bytes(&self) -> [u8; 48] {
        let mut out = [0u8; BYTES_PER_G1];
        unsafe {
            blst_p1_compress(out.as_mut_ptr(), &self.to_blst_p1());
        }
        out
    }

    fn add_or_dbl(&self, b: &Self) -> Self {
        Self(self.0 + b.0)
    }

    fn is_inf(&self) -> bool {
        let temp = &self.0;
        temp.z.is_zero()
    }

    fn is_valid(&self) -> bool {
        unsafe { blst_p1_in_g1(&self.to_blst_p1()) }
    }

    fn dbl(&self) -> Self {
        Self(self.0.double())
    }

    fn add(&self, b: &Self) -> Self {
        Self(self.0 + b.0)
    }

    fn sub(&self, b: &Self) -> Self {
        Self(self.0.sub(&b.0))
    }

    fn equals(&self, b: &Self) -> bool {
        self.0.eq(&b.0)
    }

    fn zero() -> Self {
        ArkG1::from_blst_p1(blst_p1 {
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
        })
    }

    fn add_or_dbl_assign(&mut self, b: &Self) {
        self.0 += b.0;
    }

    fn add_assign(&mut self, b: &Self) {
        self.0.add_assign(b.0);
    }

    fn dbl_assign(&mut self) {
        self.0.double_in_place();
    }
}

impl G1Mul<ArkFr> for ArkG1 {
    fn mul(&self, b: &ArkFr) -> Self {
        Self(self.0.mul(b.to_u64_arr()))
    }
}

impl G1LinComb<ArkFr, ArkFp, ArkG1Affine> for ArkG1 {
    fn g1_lincomb(
        points: &[Self],
        scalars: &[ArkFr],
        len: usize,
        precomputation: Option<&PrecomputationTable<ArkFr, Self, ArkFp, ArkG1Affine>>,
    ) -> Self {
        let mut out = Self::default();
        g1_linear_combination(&mut out, points, scalars, len, precomputation);
        out
    }
}

impl PairingVerify<ArkG1, ArkG2> for ArkG1 {
    fn verify(a1: &ArkG1, a2: &ArkG2, b1: &ArkG1, b2: &ArkG2) -> bool {
        pairings_verify(a1, a2, b1, b2)
    }
}

#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct ArkG2(pub GroupProjective<g2::Parameters>);

impl ArkG2 {
    pub fn from_blst_p2(p2: blst::blst_p2) -> Self {
        Self(blst_p2_into_pc_g2projective(&p2))
    }

    pub fn to_blst_p2(&self) -> blst::blst_p2 {
        pc_g2projective_into_blst_p2(self.0)
    }
}

impl G2 for ArkG2 {
    fn generator() -> Self {
        ArkG2::from_blst_p2(P2 {
            x: blst_fp2 {
                fp: [
                    blst_fp {
                        l: [
                            0xf5f28fa202940a10,
                            0xb3f5fb2687b4961a,
                            0xa1a893b53e2ae580,
                            0x9894999d1a3caee9,
                            0x6f67b7631863366b,
                            0x058191924350bcd7,
                        ],
                    },
                    blst_fp {
                        l: [
                            0xa5a9c0759e23f606,
                            0xaaa0c59dbccd60c3,
                            0x3bb17e18e2867806,
                            0x1b1ab6cc8541b367,
                            0xc2b6ed0ef2158547,
                            0x11922a097360edf3,
                        ],
                    },
                ],
            },
            y: blst_fp2 {
                fp: [
                    blst_fp {
                        l: [
                            0x4c730af860494c4a,
                            0x597cfa1f5e369c5a,
                            0xe7e6856caa0a635a,
                            0xbbefb5e96e0d495f,
                            0x07d3a975f0ef25a2,
                            0x0083fd8e7e80dae5,
                        ],
                    },
                    blst_fp {
                        l: [
                            0xadc0fc92df64b05d,
                            0x18aa270a2b1461dc,
                            0x86adac6a3be4eba0,
                            0x79495c4ec93da33a,
                            0xe7175850a43ccaed,
                            0x0b2bc2a163de1bf2,
                        ],
                    },
                ],
            },
            z: blst_fp2 {
                fp: [
                    blst_fp {
                        l: [
                            0x760900000002fffd,
                            0xebf4000bc40c0002,
                            0x5f48985753c758ba,
                            0x77ce585370525745,
                            0x5c071a97a256ec6d,
                            0x15f65ec3fa80e493,
                        ],
                    },
                    blst_fp {
                        l: [
                            0x0000000000000000,
                            0x0000000000000000,
                            0x0000000000000000,
                            0x0000000000000000,
                            0x0000000000000000,
                            0x0000000000000000,
                        ],
                    },
                ],
            },
        })
    }

    fn negative_generator() -> Self {
        ArkG2::from_blst_p2(P2 {
            x: blst_fp2 {
                fp: [
                    blst_fp {
                        l: [
                            0xf5f28fa202940a10,
                            0xb3f5fb2687b4961a,
                            0xa1a893b53e2ae580,
                            0x9894999d1a3caee9,
                            0x6f67b7631863366b,
                            0x058191924350bcd7,
                        ],
                    },
                    blst_fp {
                        l: [
                            0xa5a9c0759e23f606,
                            0xaaa0c59dbccd60c3,
                            0x3bb17e18e2867806,
                            0x1b1ab6cc8541b367,
                            0xc2b6ed0ef2158547,
                            0x11922a097360edf3,
                        ],
                    },
                ],
            },
            y: blst_fp2 {
                fp: [
                    blst_fp {
                        l: [
                            0x6d8bf5079fb65e61,
                            0xc52f05df531d63a5,
                            0x7f4a4d344ca692c9,
                            0xa887959b8577c95f,
                            0x4347fe40525c8734,
                            0x197d145bbaff0bb5,
                        ],
                    },
                    blst_fp {
                        l: [
                            0x0c3e036d209afa4e,
                            0x0601d8f4863f9e23,
                            0xe0832636bacc0a84,
                            0xeb2def362a476f84,
                            0x64044f659f0ee1e9,
                            0x0ed54f48d5a1caa7,
                        ],
                    },
                ],
            },
            z: blst_fp2 {
                fp: [
                    blst_fp {
                        l: [
                            0x760900000002fffd,
                            0xebf4000bc40c0002,
                            0x5f48985753c758ba,
                            0x77ce585370525745,
                            0x5c071a97a256ec6d,
                            0x15f65ec3fa80e493,
                        ],
                    },
                    blst_fp {
                        l: [
                            0x0000000000000000,
                            0x0000000000000000,
                            0x0000000000000000,
                            0x0000000000000000,
                            0x0000000000000000,
                            0x0000000000000000,
                        ],
                    },
                ],
            },
        })
    }

    #[allow(clippy::bind_instead_of_map)]
    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        bytes
            .try_into()
            .map_err(|_| {
                format!(
                    "Invalid byte length. Expected {}, got {}",
                    BYTES_PER_G2,
                    bytes.len()
                )
            })
            .and_then(|bytes: &[u8; BYTES_PER_G2]| {
                let mut tmp = blst_p2_affine::default();
                let mut g2 = blst_p2::default();
                unsafe {
                    // The uncompress routine also checks that the point is on the curve
                    if blst_p2_uncompress(&mut tmp, bytes.as_ptr()) != BLST_ERROR::BLST_SUCCESS {
                        return Err("Failed to uncompress".to_string());
                    }
                    blst_p2_from_affine(&mut g2, &tmp);
                }
                Ok(ArkG2::from_blst_p2(g2))
            })
    }

    fn to_bytes(&self) -> [u8; 96] {
        <[u8; 96]>::try_from(self.0.x.c0.0.to_bytes_le()).unwrap()
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        Self(self.0 + b.0)
    }

    fn dbl(&self) -> Self {
        Self(self.0.double())
    }

    fn sub(&self, b: &Self) -> Self {
        Self(self.0 - b.0)
    }

    fn equals(&self, b: &Self) -> bool {
        self.0.eq(&b.0)
    }
}

impl G2Mul<ArkFr> for ArkG2 {
    fn mul(&self, b: &ArkFr) -> Self {
        Self(self.0.mul(b.to_u64_arr()))
    }
}

impl Poly<ArkFr> for PolyData {
    fn new(size: usize) -> PolyData {
        Self {
            coeffs: vec![ArkFr::default(); size],
        }
    }

    fn get_coeff_at(&self, i: usize) -> ArkFr {
        self.coeffs[i]
    }

    fn set_coeff_at(&mut self, i: usize, x: &ArkFr) {
        self.coeffs[i] = *x;
    }

    fn get_coeffs(&self) -> &[ArkFr] {
        &self.coeffs
    }

    fn len(&self) -> usize {
        self.coeffs.len()
    }

    fn eval(&self, x: &ArkFr) -> ArkFr {
        if self.coeffs.is_empty() {
            return ArkFr::zero();
        } else if x.is_zero() {
            return self.coeffs[0];
        }
        let mut ret = self.coeffs[self.coeffs.len() - 1];
        let mut i = self.coeffs.len() - 2;
        loop {
            let temp = ret.mul(x);
            ret = temp.add(&self.coeffs[i]);
            if i == 0 {
                break;
            }
            i -= 1;
        }
        ret
    }

    fn scale(&mut self) {
        scale_poly(self);
    }

    fn unscale(&mut self) {
        unscale_poly(self);
    }

    fn inverse(&mut self, output_len: usize) -> Result<Self, String> {
        if output_len == 0 {
            return Err(String::from("Can't produce a zero-length result"));
        } else if self.coeffs.is_empty() {
            return Err(String::from("Can't inverse a zero-length poly"));
        } else if self.coeffs[0].is_zero() {
            return Err(String::from(
                "First coefficient of polynomial mustn't be zero",
            ));
        }

        let mut ret = PolyData {
            coeffs: vec![ArkFr::zero(); output_len],
        };
        // If the input polynomial is constant, the remainder of the series is zero
        if self.coeffs.len() == 1 {
            ret.coeffs[0] = self.coeffs[0].eucl_inverse();

            return Ok(ret);
        }

        let maxd = output_len - 1;

        // Max space for multiplications is (2 * length - 1)
        // Don't need the following as its recalculated inside
        // let scale: usize = log2_pow2(next_pow_of_2(2 * output_len - 1));
        // let fft_settings = FsFFTSettings::new(scale).unwrap();

        // To store intermediate results

        // Base case for d == 0
        ret.coeffs[0] = self.coeffs[0].eucl_inverse();
        let mut d: usize = 0;
        let mut mask: usize = 1 << log2_u64(maxd);
        while mask != 0 {
            d = 2 * d + usize::from((maxd & mask) != 0);
            mask >>= 1;

            // b.c -> tmp0 (we're using out for c)
            // tmp0.length = min_u64(d + 1, b->length + output->length - 1);
            let len_temp = (d + 1).min(self.len() + output_len - 1);
            let mut tmp0 = self.mul(&ret, len_temp).unwrap();

            // 2 - b.c -> tmp0
            for i in 0..tmp0.len() {
                tmp0.coeffs[i] = tmp0.coeffs[i].negate();
            }
            let fr_two = kzg::Fr::from_u64(2);
            tmp0.coeffs[0] = tmp0.coeffs[0].add(&fr_two);

            // c.(2 - b.c) -> tmp1;
            let tmp1 = ret.mul(&tmp0, d + 1).unwrap();

            for i in 0..tmp1.len() {
                ret.coeffs[i] = tmp1.coeffs[i];
            }
        }

        if d + 1 != output_len {
            return Err(String::from("D + 1 must be equal to output_len"));
        }

        Ok(ret)
    }

    fn div(&mut self, x: &Self) -> Result<Self, String> {
        if x.len() >= self.len() || x.len() < 128 {
            self.long_div(x)
        } else {
            self.fast_div(x)
        }
    }

    fn long_div(&mut self, divisor: &Self) -> Result<Self, String> {
        if divisor.coeffs.is_empty() {
            return Err(String::from("Can't divide by zero"));
        } else if divisor.coeffs[divisor.coeffs.len() - 1].is_zero() {
            return Err(String::from("Highest coefficient must be non-zero"));
        }

        let out_length = self.poly_quotient_length(divisor);
        if out_length == 0 {
            return Ok(PolyData { coeffs: vec![] });
        }

        // Special case for divisor.len() == 2
        if divisor.len() == 2 {
            let divisor_0 = divisor.coeffs[0];
            let divisor_1 = divisor.coeffs[1];

            let mut out_coeffs = Vec::from(&self.coeffs[1..]);
            for i in (1..out_length).rev() {
                out_coeffs[i] = out_coeffs[i].div(&divisor_1).unwrap();

                let tmp = out_coeffs[i].mul(&divisor_0);
                out_coeffs[i - 1] = out_coeffs[i - 1].sub(&tmp);
            }

            out_coeffs[0] = out_coeffs[0].div(&divisor_1).unwrap();

            Ok(PolyData { coeffs: out_coeffs })
        } else {
            let mut out: PolyData = PolyData {
                coeffs: vec![ArkFr::default(); out_length],
            };

            let mut a_pos = self.len() - 1;
            let b_pos = divisor.len() - 1;
            let mut diff = a_pos - b_pos;

            let mut a = self.coeffs.clone();

            while diff > 0 {
                out.coeffs[diff] = a[a_pos].div(&divisor.coeffs[b_pos]).unwrap();

                for i in 0..(b_pos + 1) {
                    let tmp = out.coeffs[diff].mul(&divisor.coeffs[i]);
                    a[diff + i] = a[diff + i].sub(&tmp);
                }

                diff -= 1;
                a_pos -= 1;
            }

            out.coeffs[0] = a[a_pos].div(&divisor.coeffs[b_pos]).unwrap();
            Ok(out)
        }
    }

    fn fast_div(&mut self, divisor: &Self) -> Result<Self, String> {
        if divisor.coeffs.is_empty() {
            return Err(String::from("Cant divide by zero"));
        } else if divisor.coeffs[divisor.coeffs.len() - 1].is_zero() {
            return Err(String::from("Highest coefficient must be non-zero"));
        }

        let m: usize = self.len() - 1;
        let n: usize = divisor.len() - 1;

        // If the divisor is larger than the dividend, the result is zero-length
        if n > m {
            return Ok(PolyData { coeffs: Vec::new() });
        }

        // Special case for divisor.length == 1 (it's a constant)
        if divisor.len() == 1 {
            let mut out = PolyData {
                coeffs: vec![ArkFr::zero(); self.len()],
            };
            for i in 0..out.len() {
                out.coeffs[i] = self.coeffs[i].div(&divisor.coeffs[0]).unwrap();
            }
            return Ok(out);
        }

        let mut a_flip = self.flip().unwrap();
        let mut b_flip = divisor.flip().unwrap();

        let inv_b_flip = b_flip.inverse(m - n + 1).unwrap();
        let q_flip = a_flip.mul(&inv_b_flip, m - n + 1).unwrap();

        let out = q_flip.flip().unwrap();
        Ok(out)
    }

    fn mul_direct(&mut self, multiplier: &Self, output_len: usize) -> Result<Self, String> {
        if self.len() == 0 || multiplier.len() == 0 {
            return Ok(PolyData::new(0));
        }

        let a_degree = self.len() - 1;
        let b_degree = multiplier.len() - 1;

        let mut ret = PolyData {
            coeffs: vec![kzg::Fr::zero(); output_len],
        };

        // Truncate the output to the length of the output polynomial
        for i in 0..(a_degree + 1) {
            let mut j = 0;
            while (j <= b_degree) && ((i + j) < output_len) {
                let tmp = self.coeffs[i].mul(&multiplier.coeffs[j]);
                let tmp = ret.coeffs[i + j].add(&tmp);
                ret.coeffs[i + j] = tmp;

                j += 1;
            }
        }

        Ok(ret)
    }
}

impl FFTSettingsPoly<ArkFr, PolyData, LFFTSettings> for LFFTSettings {
    fn poly_mul_fft(
        a: &PolyData,
        b: &PolyData,
        len: usize,
        _fs: Option<&LFFTSettings>,
    ) -> Result<PolyData, String> {
        b.mul_fft(a, len)
    }
}

impl Default for LFFTSettings {
    fn default() -> Self {
        Self {
            max_width: 0,
            root_of_unity: ArkFr::zero(),
            brp_roots_of_unity: Vec::new(),
            reverse_roots_of_unity: Vec::new(),
            roots_of_unity: Vec::new(),
        }
    }
}

impl FFTSettings<ArkFr> for LFFTSettings {
    fn new(scale: usize) -> Result<LFFTSettings, String> {
        if scale >= SCALE2_ROOT_OF_UNITY.len() {
            return Err(String::from(
                "Scale is expected to be within root of unity matrix row size",
            ));
        }

        let max_width: usize = 1 << scale;
        let root_of_unity = ArkFr::from_u64_arr(&SCALE2_ROOT_OF_UNITY[scale]);

        let roots_of_unity = expand_root_of_unity(&root_of_unity, max_width)?;

        let mut brp_roots_of_unity = roots_of_unity.clone();
        brp_roots_of_unity.pop();
        reverse_bit_order(&mut brp_roots_of_unity)?;
        let mut reverse_roots_of_unity = roots_of_unity.clone();
        reverse_roots_of_unity.reverse();

        // let expanded_roots_of_unity = expand_root_of_unity(&root_of_unity, max_width)?;
        // let mut reverse_roots_of_unity = expanded_roots_of_unity.clone();
        // reverse_roots_of_unity.reverse();

        // let mut roots_of_unity = expanded_roots_of_unity.clone();
        // roots_of_unity.pop();
        // reverse_bit_order(&mut roots_of_unity)?;

        Ok(LFFTSettings {
            max_width,
            root_of_unity,
            brp_roots_of_unity,
            reverse_roots_of_unity,
            roots_of_unity,
        })
    }

    fn get_max_width(&self) -> usize {
        self.max_width
    }

    fn get_brp_roots_of_unity(&self) -> &[ArkFr] {
        &self.brp_roots_of_unity
    }
    fn get_brp_roots_of_unity_at(&self, i: usize) -> ArkFr {
        self.brp_roots_of_unity[i]
    }

    fn get_reverse_roots_of_unity_at(&self, i: usize) -> ArkFr {
        self.reverse_roots_of_unity[i]
    }

    fn get_reversed_roots_of_unity(&self) -> &[ArkFr] {
        &self.reverse_roots_of_unity
    }

    fn get_roots_of_unity_at(&self, i: usize) -> ArkFr {
        self.roots_of_unity[i]
    }

    fn get_roots_of_unity(&self) -> &[ArkFr] {
        &self.roots_of_unity
    }
}

fn toeplitz_part_1(
    field_elements_per_ext_blob: usize,
    output: &mut [ArkG1],
    x: &[ArkG1],
    s: &LFFTSettings,
) -> Result<(), String> {
    let n = x.len();
    let n2 = n * 2;
    let mut x_ext = vec![ArkG1::identity(); n2];

    x_ext[..n].copy_from_slice(x);

    let x_ext = &x_ext[..];

    /* Ensure the length is valid */
    if x_ext.len() > field_elements_per_ext_blob || !x_ext.len().is_power_of_two() {
        return Err("Invalid input size".to_string());
    }

    let roots_stride = field_elements_per_ext_blob / x_ext.len();
    fft_g1_fast(output, x_ext, 1, &s.roots_of_unity, roots_stride);

    Ok(())
}

impl KZGSettings<ArkFr, ArkG1, ArkG2, LFFTSettings, PolyData, ArkFp, ArkG1Affine> for LKZGSettings {
    fn new(
        g1_monomial: &[ArkG1],
        g1_lagrange_brp: &[ArkG1],
        g2_monomial: &[ArkG2],
        fft_settings: &LFFTSettings,
        cell_size: usize,
    ) -> Result<LKZGSettings, String> {
        if g1_monomial.len() != g1_lagrange_brp.len() {
            return Err("G1 point length mismatch".to_string());
        }

        let field_elements_per_blob = g1_monomial.len();
        let field_elements_per_ext_blob = field_elements_per_blob * 2;

        let n = field_elements_per_ext_blob / 2;
        let k = n / cell_size;
        let k2 = 2 * k;

        let mut points = vec![ArkG1::default(); k2];
        let mut x = vec![ArkG1::default(); k];
        let mut x_ext_fft_columns = vec![vec![ArkG1::default(); cell_size]; k2];

        for offset in 0..cell_size {
            let start = n - cell_size - 1 - offset;
            for (i, p) in x.iter_mut().enumerate().take(k - 1) {
                let j = start - i * cell_size;
                *p = g1_monomial[j];
            }
            x[k - 1] = ArkG1::identity();

            toeplitz_part_1(field_elements_per_ext_blob, &mut points, &x, fft_settings)?;

            for row in 0..k2 {
                x_ext_fft_columns[row][offset] = points[row];
            }
        }

        Ok(Self {
            g1_values_monomial: g1_monomial.to_vec(),
            g1_values_lagrange_brp: g1_lagrange_brp.to_vec(),
            g2_values_monomial: g2_monomial.to_vec(),
            fs: fft_settings.clone(),
            x_ext_fft_columns,
            precomputation: {
                #[cfg(feature = "sppark")]
                {
                    use ark_bls12_381::G1Affine;
                    let points =
                        kzg::msm::msm_impls::batch_convert::<ArkG1, ArkFp, ArkG1Affine>(secret_g1);
                    let points = unsafe {
                        alloc::slice::from_raw_parts(
                            points.as_ptr() as *const G1Affine,
                            points.len(),
                        )
                    };
                    let prepared = rust_kzg_arkworks3_sppark::prepare_multi_scalar_mult(points);
                    Some(Arc::new(PrecomputationTable::from_ptr(prepared)))
                }

                #[cfg(feature = "sppark_wlc")]
                {
                    let affines =
                        kzg::msm::msm_impls::batch_convert::<ArkG1, ArkFp, ArkG1Affine>(&secret_g1);
                    let affines = unsafe {
                        alloc::slice::from_raw_parts(
                            affines.as_ptr() as *const G1Affine,
                            secret_g1.len(),
                        )
                    };

                    Some(Arc::new(PrecomputationTable::from_ptr(
                        rust_kzg_arkworks3_sppark_wlc::multi_scalar_mult_init(affines).context,
                    )))
                }

                #[cfg(not(any(feature = "sppark", feature = "sppark_wlc")))]
                {
                    precompute(g1_lagrange_brp).ok().flatten().map(Arc::new)
                }
            },
            cell_size,
        })
    }

    fn commit_to_poly(&self, p: &PolyData) -> Result<ArkG1, String> {
        if p.coeffs.len() > self.g1_values_lagrange_brp.len() {
            return Err(String::from("Polynomial is longer than secret g1"));
        }

        let mut out = ArkG1::default();
        g1_linear_combination(
            &mut out,
            &self.g1_values_lagrange_brp,
            &p.coeffs,
            p.coeffs.len(),
            self.get_precomputation(),
        );

        Ok(out)
    }

    fn compute_proof_single(&self, p: &PolyData, x: &ArkFr) -> Result<ArkG1, String> {
        if p.coeffs.is_empty() {
            return Err(String::from("Polynomial must not be empty"));
        }

        // `-(x0^n)`, where `n` is `1`
        let divisor_0 = x.negate();

        // Calculate `q = p / (x^n - x0^n)` for our reduced case (see `compute_proof_multi` for
        // generic implementation)
        let mut out_coeffs = Vec::from(&p.coeffs[1..]);
        for i in (1..out_coeffs.len()).rev() {
            let tmp = out_coeffs[i].mul(&divisor_0);
            out_coeffs[i - 1] = out_coeffs[i - 1].sub(&tmp);
        }

        let q = PolyData { coeffs: out_coeffs };
        let ret = self.commit_to_poly(&q)?;
        Ok(ret)
        // Ok(compute_single(p, x, self))
    }

    fn check_proof_single(
        &self,
        com: &ArkG1,
        proof: &ArkG1,
        x: &ArkFr,
        y: &ArkFr,
    ) -> Result<bool, String> {
        let x_g2: ArkG2 = ArkG2::generator().mul(x);
        let s_minus_x: ArkG2 = self.g2_values_monomial[1].sub(&x_g2);
        let y_g1 = ArkG1::generator().mul(y);
        let commitment_minus_y: ArkG1 = com.sub(&y_g1);

        Ok(pairings_verify(
            &commitment_minus_y,
            &ArkG2::generator(),
            proof,
            &s_minus_x,
        ))
    }

    fn compute_proof_multi(&self, p: &PolyData, x: &ArkFr, n: usize) -> Result<ArkG1, String> {
        if p.coeffs.is_empty() {
            return Err(String::from("Polynomial must not be empty"));
        }

        if !n.is_power_of_two() {
            return Err(String::from("n must be a power of two"));
        }

        // Construct x^n - x0^n = (x - x0.w^0)(x - x0.w^1)...(x - x0.w^(n-1))
        let mut divisor = PolyData {
            coeffs: Vec::with_capacity(n + 1),
        };

        // -(x0^n)
        let x_pow_n = x.pow(n);

        divisor.coeffs.push(x_pow_n.negate());

        // Zeros
        for _ in 1..n {
            divisor.coeffs.push(kzg::Fr::zero());
        }

        // x^n
        divisor.coeffs.push(kzg::Fr::one());

        let mut new_polina = p.clone();

        // Calculate q = p / (x^n - x0^n)
        // let q = p.div(&divisor).unwrap();
        let q = new_polina.div(&divisor)?;
        let ret = self.commit_to_poly(&q)?;
        Ok(ret)
    }

    fn check_proof_multi(
        &self,
        com: &ArkG1,
        proof: &ArkG1,
        x: &ArkFr,
        ys: &[ArkFr],
        n: usize,
    ) -> Result<bool, String> {
        if !n.is_power_of_two() {
            return Err(String::from("n is not a power of two"));
        }

        // Interpolate at a coset.
        let mut interp = PolyData {
            coeffs: self.fs.fft_fr(ys, true)?,
        };

        let inv_x = x.inverse(); // Not euclidean?
        let mut inv_x_pow = inv_x;
        for i in 1..n {
            interp.coeffs[i] = interp.coeffs[i].mul(&inv_x_pow);
            inv_x_pow = inv_x_pow.mul(&inv_x);
        }

        // [x^n]_2
        let x_pow = inv_x_pow.inverse();

        let xn2 = ArkG2::generator().mul(&x_pow);

        // [s^n - x^n]_2
        let xn_minus_yn = self.g2_values_monomial[n].sub(&xn2);

        // [interpolation_polynomial(s)]_1
        let is1 = self.commit_to_poly(&interp).unwrap();

        // [commitment - interpolation_polynomial(s)]_1 = [commit]_1 - [interpolation_polynomial(s)]_1
        let commit_minus_interp = com.sub(&is1);

        let ret = pairings_verify(
            &commit_minus_interp,
            &ArkG2::generator(),
            proof,
            &xn_minus_yn,
        );

        Ok(ret)
    }

    fn get_roots_of_unity_at(&self, i: usize) -> ArkFr {
        self.fs.get_roots_of_unity_at(i)
    }

    fn get_fft_settings(&self) -> &LFFTSettings {
        &self.fs
    }

    fn get_g1_lagrange_brp(&self) -> &[ArkG1] {
        &self.g1_values_lagrange_brp
    }

    fn get_g1_monomial(&self) -> &[ArkG1] {
        &self.g1_values_monomial
    }
    fn get_g2_monomial(&self) -> &[ArkG2] {
        &self.g2_values_monomial
    }
    fn get_x_ext_fft_column(&self, index: usize) -> &[ArkG1] {
        &self.x_ext_fft_columns[index]
    }

    fn get_precomputation(&self) -> Option<&PrecomputationTable<ArkFr, ArkG1, ArkFp, ArkG1Affine>> {
        self.precomputation.as_ref().map(|v| v.as_ref())
    }

    fn get_cell_size(&self) -> usize {
        self.cell_size
    }
}

type ArkFpInt = <ark_bls12_381::g1::Parameters as ModelParameters>::BaseField;
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct ArkFp(pub ArkFpInt);

impl G1Fp for ArkFp {
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    fn set_zero(&mut self) {
        self.0.set_zero();
    }

    fn is_one(&self) -> bool {
        self.0.is_one()
    }

    fn set_one(&mut self) {
        self.0.set_one();
    }

    fn inverse(&self) -> Option<Self> {
        Some(Self(self.0.inverse().unwrap()))
    }

    fn square(&self) -> Self {
        Self(self.0.square())
    }

    fn double(&self) -> Self {
        Self(self.0.double())
    }

    fn from_underlying_arr(arr: &[u64; 6]) -> Self {
        let mut default = ArkFpInt::default();
        default.0 .0 = *arr;
        Self(default)
    }

    fn neg_assign(&mut self) {
        self.0 = -self.0;
    }

    fn mul_assign_fp(&mut self, b: &Self) {
        self.0 *= b.0;
    }

    fn sub_assign_fp(&mut self, b: &Self) {
        self.0 -= b.0;
    }

    fn add_assign_fp(&mut self, b: &Self) {
        self.0 += b.0;
    }

    fn zero() -> Self {
        Self(ArkFpInt::zero())
    }

    fn one() -> Self {
        Self(ArkFpInt::one())
    }

    fn bls12_381_rx_p() -> Self {
        Self(blst_fp_into_pc_fq(&blst_fp {
            l: [
                8505329371266088957,
                17002214543764226050,
                6865905132761471162,
                8632934651105793861,
                6631298214892334189,
                1582556514881692819,
            ],
        }))
    }
}

impl G1GetFp<ArkFp> for ArkG1 {
    fn x(&self) -> &ArkFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&self.0.x)
        }
    }

    fn y(&self) -> &ArkFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&self.0.y)
        }
    }

    fn z(&self) -> &ArkFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&self.0.z)
        }
    }

    fn x_mut(&mut self) -> &mut ArkFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&mut self.0.x)
        }
    }

    fn y_mut(&mut self) -> &mut ArkFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&mut self.0.y)
        }
    }

    fn z_mut(&mut self) -> &mut ArkFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&mut self.0.z)
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct ArkG1Affine {
    pub aff: G1Affine,
}

impl G1AffineTrait<ArkG1, ArkFp> for ArkG1Affine {
    fn into_affine(g1: &ArkG1) -> Self {
        Self {
            aff: g1.0.into_affine(),
        }
    }

    fn into_affines(g1: &[ArkG1]) -> Vec<Self> {
        let ark_points: &[GroupProjective<g1::Parameters>] = unsafe { core::mem::transmute(g1) };
        let ark_points = GroupProjective::batch_normalization_into_affine(ark_points);
        unsafe { core::mem::transmute(ark_points) }
    }

    fn into_affines_loc(out: &mut [Self], g1: &[ArkG1]) {
        out.copy_from_slice(&Self::into_affines(g1));
    }

    fn to_proj(&self) -> ArkG1 {
        ArkG1(self.aff.into_projective())
    }

    fn x(&self) -> &ArkFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&self.aff.x)
        }
    }

    fn y(&self) -> &ArkFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&self.aff.y)
        }
    }

    fn is_infinity(&self) -> bool {
        self.aff.infinity
    }

    fn is_zero(&self) -> bool {
        self.aff.is_zero()
    }

    fn zero() -> Self {
        Self {
            aff: G1Affine::new(ArkFp::zero().0, ArkFp::zero().0, true),
        }
    }

    fn x_mut(&mut self) -> &mut ArkFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&mut self.aff.x)
        }
    }

    fn y_mut(&mut self) -> &mut ArkFp {
        unsafe {
            // Transmute safe due to repr(C) on FsFp
            core::mem::transmute(&mut self.aff.y)
        }
    }
}

pub struct ArkG1ProjAddAffine;
impl G1ProjAddAffine<ArkG1, ArkFp, ArkG1Affine> for ArkG1ProjAddAffine {
    fn add_assign_affine(proj: &mut ArkG1, aff: &ArkG1Affine) {
        proj.0.add_assign_mixed(&aff.aff);
    }

    fn add_or_double_assign_affine(proj: &mut ArkG1, aff: &ArkG1Affine) {
        proj.0.add_assign_mixed(&aff.aff);
    }
}

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;

use constantine::ctt_codec_ecc_status;
use kzg::eip_4844::BYTES_PER_G2;
#[cfg(feature = "rand")]
use kzg::Fr;
use kzg::{G2Mul, G2};

use crate::consts::{G2_GENERATOR, G2_NEGATIVE_GENERATOR};
use crate::types::fr::CtFr;

use constantine_sys::{
    bls12_381_fp, bls12_381_fp2, bls12_381_g2_aff, bls12_381_g2_jac,
    ctt_bls12_381_g2_jac_from_affine,
};

use constantine_sys as constantine;

use kzg::eth::c_bindings::{blst_fp, blst_fp2, blst_p2};

#[derive(Default, Clone, Copy)]
pub struct CtG2(pub bls12_381_g2_jac);

impl CtG2 {
    pub const fn from_blst_p2(p2: blst_p2) -> Self {
        unsafe {
            Self(bls12_381_g2_jac {
                x: bls12_381_fp2 {
                    c: [
                        bls12_381_fp {
                            limbs: core::mem::transmute::<[u64; 6], [usize; 6]>(p2.x.fp[0].l),
                        },
                        bls12_381_fp {
                            limbs: core::mem::transmute::<[u64; 6], [usize; 6]>(p2.x.fp[1].l),
                        },
                    ],
                },
                y: bls12_381_fp2 {
                    c: [
                        bls12_381_fp {
                            limbs: core::mem::transmute::<[u64; 6], [usize; 6]>(p2.y.fp[0].l),
                        },
                        bls12_381_fp {
                            limbs: core::mem::transmute::<[u64; 6], [usize; 6]>(p2.y.fp[1].l),
                        },
                    ],
                },
                z: bls12_381_fp2 {
                    c: [
                        bls12_381_fp {
                            limbs: core::mem::transmute::<[u64; 6], [usize; 6]>(p2.z.fp[0].l),
                        },
                        bls12_381_fp {
                            limbs: core::mem::transmute::<[u64; 6], [usize; 6]>(p2.z.fp[1].l),
                        },
                    ],
                },
            })
        }
    }

    pub const fn to_blst_p2(&self) -> blst_p2 {
        unsafe {
            blst_p2 {
                x: blst_fp2 {
                    fp: [
                        blst_fp {
                            l: core::mem::transmute::<[usize; 6], [u64; 6]>(self.0.x.c[0].limbs),
                        },
                        blst_fp {
                            l: core::mem::transmute::<[usize; 6], [u64; 6]>(self.0.x.c[1].limbs),
                        },
                    ],
                },
                y: blst_fp2 {
                    fp: [
                        blst_fp {
                            l: core::mem::transmute::<[usize; 6], [u64; 6]>(self.0.y.c[0].limbs),
                        },
                        blst_fp {
                            l: core::mem::transmute::<[usize; 6], [u64; 6]>(self.0.y.c[1].limbs),
                        },
                    ],
                },
                z: blst_fp2 {
                    fp: [
                        blst_fp {
                            l: core::mem::transmute::<[usize; 6], [u64; 6]>(self.0.z.c[0].limbs),
                        },
                        blst_fp {
                            l: core::mem::transmute::<[usize; 6], [u64; 6]>(self.0.z.c[1].limbs),
                        },
                    ],
                },
            }
        }
    }
}

impl G2Mul<CtFr> for CtG2 {
    fn mul(&self, b: &CtFr) -> Self {
        let mut result = *self;
        unsafe {
            constantine::ctt_bls12_381_g2_jac_scalar_mul_fr_coef(&mut result.0, &b.0);
        }
        result
    }
}

impl G2 for CtG2 {
    fn generator() -> Self {
        G2_GENERATOR
    }

    fn negative_generator() -> Self {
        G2_NEGATIVE_GENERATOR
    }

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
                let mut tmp = bls12_381_g2_aff::default();
                let mut g2 = bls12_381_g2_jac::default();
                unsafe {
                    // The uncompress routine also checks that the point is on the curve
                    let res = constantine::ctt_bls12_381_deserialize_g2_compressed(
                        &mut tmp,
                        bytes.as_ptr(),
                    );
                    if res != ctt_codec_ecc_status::cttCodecEcc_Success
                        && res != ctt_codec_ecc_status::cttCodecEcc_PointAtInfinity
                    {
                        return Err("Failed to uncompress".to_string());
                    }
                    ctt_bls12_381_g2_jac_from_affine(&mut g2, &tmp);
                }
                Ok(CtG2(g2))
            })
    }

    fn to_bytes(&self) -> [u8; 96] {
        let mut out = [0u8; BYTES_PER_G2];
        let mut tmp = bls12_381_g2_aff::default();
        unsafe {
            constantine::ctt_bls12_381_g2_jac_affine(&mut tmp, &self.0);
            let _ = constantine::ctt_bls12_381_serialize_g2_compressed(out.as_mut_ptr(), &tmp);
        }
        out
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        let mut result = self.0;
        unsafe {
            constantine::ctt_bls12_381_g2_jac_add_in_place(&mut result, &b.0);
        }
        Self(result)
    }

    fn dbl(&self) -> Self {
        let mut result = bls12_381_g2_jac::default();
        unsafe {
            constantine::ctt_bls12_381_g2_jac_double(&mut result, &self.0);
        }
        Self(result)
    }

    fn sub(&self, b: &Self) -> Self {
        let mut bneg: bls12_381_g2_jac = b.0;
        let mut result = self.0;
        unsafe {
            constantine::ctt_bls12_381_g2_jac_neg_in_place(&mut bneg);
            constantine::ctt_bls12_381_g2_jac_add_in_place(&mut result, &bneg);
        }
        Self(result)
    }

    fn equals(&self, b: &Self) -> bool {
        unsafe { constantine::ctt_bls12_381_g2_jac_is_eq(&self.0, &b.0) != 0 }
    }
}

impl CtG2 {
    pub(crate) fn _from_xyz(x: bls12_381_fp2, y: bls12_381_fp2, z: bls12_381_fp2) -> Self {
        CtG2(bls12_381_g2_jac { x, y, z })
    }

    #[cfg(feature = "rand")]
    pub fn rand() -> Self {
        let result: CtG2 = G2_GENERATOR;
        result.mul(&CtFr::rand())
    }
}

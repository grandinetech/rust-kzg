use crate::consts::{
    G1_GENERATOR, G1_IDENTITY, G1_NEGATIVE_GENERATOR, G2_GENERATOR, G2_NEGATIVE_GENERATOR,
    SCALE2_ROOT_OF_UNITY,
};
use crate::fft_g1::g1_linear_combination;
use crate::kzg_proofs::{
    expand_root_of_unity, pairings_verify, FFTSettings as ZFFTSettings, KZGSettings as ZKZGSettings,
};
use crate::poly::PolyData;
use crate::utils::{
    blst_fr_into_pc_fr, blst_p1_into_pc_g1projective, blst_p2_into_pc_g2projective,
    fft_settings_to_rust, pc_fr_into_blst_fr, pc_g1projective_into_blst_p1,
    pc_g2projective_into_blst_p2, PRECOMPUTATION_TABLES,
};
use bls12_381::{Fp, G1Affine, G1Projective, G2Affine, G2Projective, Scalar, MODULUS, R2};
use blst::{blst_fr, blst_p1};
use ff::Field;
use kzg::common_utils::reverse_bit_order;
use kzg::eip_4844::{BYTES_PER_FIELD_ELEMENT, BYTES_PER_G1, BYTES_PER_G2};
use kzg::eth::c_bindings::CKZGSettings;
use kzg::msm::precompute::{precompute, PrecomputationTable};
use kzg::{eth, G1Affine as G1AffineTrait};
use kzg::{
    FFTFr, FFTSettings, Fr as KzgFr, G1Fp, G1GetFp, G1LinComb, G1Mul, G1ProjAddAffine, G2Mul,
    KZGSettings, PairingVerify, Poly, Scalar256, G1, G2,
};
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};
use std::sync::Arc;

use ff::derive::sbb;
use subtle::{Choice, ConstantTimeEq, CtOption};

fn to_scalar(zfr: &ZFr) -> Scalar {
    zfr.fr
}

fn bigint_check_mod_256(a: &[u64; 4]) -> bool {
    let (_, overflow) = a[0].overflowing_sub(MODULUS.0[0]);
    let (_, overflow) = a[1].overflowing_sub(MODULUS.0[1] + overflow as u64);
    let (_, overflow) = a[2].overflowing_sub(MODULUS.0[2] + overflow as u64);
    let (_, overflow) = a[3].overflowing_sub(MODULUS.0[3] + overflow as u64);
    overflow
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct ZFr {
    pub fr: Scalar,
}

impl ZFr {
    pub fn from_blst_fr(fr: blst_fr) -> Self {
        Self {
            fr: blst_fr_into_pc_fr(fr),
        }
    }
    pub fn to_blst_fr(&self) -> blst_fr {
        pc_fr_into_blst_fr(self.fr)
    }

    pub fn converter(points: &[ZFr]) -> Vec<Scalar> {
        let mut result = Vec::new();

        for zg1 in points {
            result.push(zg1.fr);
        }
        result
    }
}

impl KzgFr for ZFr {
    fn null() -> Self {
        Self {
            fr: Scalar([u64::MAX, u64::MAX, u64::MAX, u64::MAX]),
        }
    }
    fn zero() -> Self {
        Self::from_u64(0)
    }

    fn one() -> Self {
        Self::from_u64(1)
    }

    #[cfg(feature = "rand")]
    fn rand() -> Self {
        let rng = rand::thread_rng();
        let rusult = ff::Field::random(rng);
        Self { fr: rusult }
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

                tmp.0[0] = u64::from_be_bytes(<[u8; 8]>::try_from(&bytes[0..8]).unwrap());
                tmp.0[1] = u64::from_be_bytes(<[u8; 8]>::try_from(&bytes[8..16]).unwrap());
                tmp.0[2] = u64::from_be_bytes(<[u8; 8]>::try_from(&bytes[16..24]).unwrap());
                tmp.0[3] = u64::from_be_bytes(<[u8; 8]>::try_from(&bytes[24..32]).unwrap());

                // Try to subtract the modulus
                let (_, borrow) = sbb(tmp.0[0], MODULUS.0[0], 0);
                let (_, borrow) = sbb(tmp.0[1], MODULUS.0[1], borrow);
                let (_, borrow) = sbb(tmp.0[2], MODULUS.0[2], borrow);
                let (_, _borrow) = sbb(tmp.0[3], MODULUS.0[3], borrow);
                let mut tmp2 = Scalar::default();

                tmp2.0[0] = tmp.0[3];
                tmp2.0[1] = tmp.0[2];
                tmp2.0[2] = tmp.0[1];
                tmp2.0[3] = tmp.0[0];

                let is_zero: bool = tmp2.is_zero().into();
                if !is_zero && !bigint_check_mod_256(&tmp2.0) {
                    return Err("Invalid scalar".to_string());
                }

                tmp2 *= &R2;
                Ok(Self { fr: tmp2 })
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
                let mut tmp = Scalar([0, 0, 0, 0]);

                tmp.0[0] = u64::from_be_bytes(<[u8; 8]>::try_from(&bytes[0..8]).unwrap());
                tmp.0[1] = u64::from_be_bytes(<[u8; 8]>::try_from(&bytes[8..16]).unwrap());
                tmp.0[2] = u64::from_be_bytes(<[u8; 8]>::try_from(&bytes[16..24]).unwrap());
                tmp.0[3] = u64::from_be_bytes(<[u8; 8]>::try_from(&bytes[24..32]).unwrap());

                // Try to subtract the modulus
                let (_, borrow) = sbb(tmp.0[0], MODULUS.0[0], 0);
                let (_, borrow) = sbb(tmp.0[1], MODULUS.0[1], borrow);
                let (_, borrow) = sbb(tmp.0[2], MODULUS.0[2], borrow);
                let (_, _borrow) = sbb(tmp.0[3], MODULUS.0[3], borrow);
                let mut tmp2 = Scalar::default();

                tmp2.0[0] = tmp.0[3];
                tmp2.0[1] = tmp.0[2];
                tmp2.0[2] = tmp.0[1];
                tmp2.0[3] = tmp.0[0];

                tmp2 *= &R2;
                Self { fr: tmp2 }
            })
    }

    fn from_hex(hex: &str) -> Result<Self, String> {
        let bytes = hex::decode(&hex[2..]).unwrap();
        Self::from_bytes(&bytes)
    }

    fn from_u64_arr(u: &[u64; 4]) -> Self {
        Self {
            fr: Scalar::from_raw(*u),
        }
    }

    fn from_u64(val: u64) -> Self {
        Self {
            fr: Scalar::from(val),
        }
    }

    fn to_bytes(&self) -> [u8; 32] {
        let scalar = self.fr;
        let tmp = Scalar::montgomery_reduce(
            scalar.0[0],
            scalar.0[1],
            scalar.0[2],
            scalar.0[3],
            0,
            0,
            0,
            0,
        );
        let mut res = [0; 32];
        res[0..8].copy_from_slice(&tmp.0[3].to_be_bytes());
        res[8..16].copy_from_slice(&tmp.0[2].to_be_bytes());
        res[16..24].copy_from_slice(&tmp.0[1].to_be_bytes());
        res[24..32].copy_from_slice(&tmp.0[0].to_be_bytes());
        res
    }

    //testuoti
    fn to_u64_arr(&self) -> [u64; 4] {
        let bytes = self.to_bytes();
        [
            u64::from_be_bytes(bytes[24..32].try_into().unwrap()),
            u64::from_be_bytes(bytes[16..24].try_into().unwrap()),
            u64::from_be_bytes(bytes[8..16].try_into().unwrap()),
            u64::from_be_bytes(bytes[0..8].try_into().unwrap()),
        ]
    }

    fn is_one(&self) -> bool {
        self.fr.ct_eq(&ZFr::one().fr).unwrap_u8() == 1
    }

    fn is_zero(&self) -> bool {
        self.fr.is_zero().unwrap_u8() == 1
    }

    fn is_null(&self) -> bool {
        self.fr.ct_eq(&ZFr::null().fr).unwrap_u8() == 1
    }

    fn sqr(&self) -> Self {
        Self {
            fr: self.fr.square(),
        }
    }

    fn mul(&self, b: &Self) -> Self {
        Self {
            fr: Scalar::mul(&to_scalar(self), &to_scalar(b)),
        }
    }

    fn add(&self, b: &Self) -> Self {
        Self { fr: self.fr + b.fr }
    }

    fn sub(&self, b: &Self) -> Self {
        Self { fr: self.fr - b.fr }
    }

    fn eucl_inverse(&self) -> Self {
        Self {
            fr: self.fr.invert().unwrap(),
        }
    }

    fn negate(&self) -> Self {
        Self { fr: self.fr.neg() }
    }

    fn inverse(&self) -> Self {
        Self {
            fr: self.fr.invert().unwrap(),
        }
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
        if <ZFr>::is_zero(b) {
            return Err("Cannot divide by zero".to_string());
        }
        let tmp = b.eucl_inverse();
        let out = self.mul(&tmp);
        Ok(out)
    }

    fn equals(&self, b: &Self) -> bool {
        self.fr == b.fr
    }

    fn to_scalar(&self) -> Scalar256 {
        let tmp = Scalar::montgomery_reduce(
            self.fr.0[0],
            self.fr.0[1],
            self.fr.0[2],
            self.fr.0[3],
            0,
            0,
            0,
            0,
        );
        Scalar256::from_u64(tmp.0)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct ZFp(pub Fp);
impl G1Fp for ZFp {
    fn zero() -> Self {
        Self(Fp::zero())
    }

    fn one() -> Self {
        Self(Fp::one())
    }

    fn bls12_381_rx_p() -> Self {
        Self(Fp([
            8505329371266088957,
            17002214543764226050,
            6865905132761471162,
            8632934651105793861,
            6631298214892334189,
            1582556514881692819,
        ]))
    }

    fn inverse(&self) -> Option<Self> {
        self.0.invert().map(Self).into()
    }

    fn square(&self) -> Self {
        Self(self.0.square())
    }

    fn double(&self) -> Self {
        Self(self.0.add(&self.0))
    }

    fn from_underlying_arr(arr: &[u64; 6]) -> Self {
        Self(Fp(*arr))
    }

    fn neg_assign(&mut self) {
        self.0 = self.0.neg();
    }

    fn mul_assign_fp(&mut self, b: &Self) {
        self.0.mul_assign(b.0);
    }

    fn sub_assign_fp(&mut self, b: &Self) {
        self.0.sub_assign(b.0);
    }

    fn add_assign_fp(&mut self, b: &Self) {
        self.0.add_assign(b.0);
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct ZG1 {
    pub proj: G1Projective,
}

impl ZG1 {
    pub const fn from_blst_p1(p1: blst_p1) -> Self {
        Self {
            proj: blst_p1_into_pc_g1projective(&p1),
        }
    }

    pub const fn to_blst_p1(&self) -> blst_p1 {
        pc_g1projective_into_blst_p1(self.proj)
    }
    pub const fn from_g1_projective(proj: G1Projective) -> Self {
        Self { proj }
    }

    fn affine_to_projective(p: G1Affine) -> Self {
        Self {
            proj: G1Projective::from(&p),
        }
    }
    pub fn converter(points: &[ZG1]) -> Vec<G1Projective> {
        let mut result = Vec::new();

        for zg1 in points {
            result.push(zg1.proj);
        }
        result
    }
}

impl From<blst_p1> for ZG1 {
    fn from(p1: blst_p1) -> Self {
        let proj = blst_p1_into_pc_g1projective(&p1);
        Self { proj }
    }
}

impl G1 for ZG1 {
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
        let mut rng = rand::thread_rng();
        Self {
            proj: G1Projective::random(&mut rng),
        }
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
                let affine: CtOption<G1Affine> = G1Affine::from_compressed(bytes);
                match affine.into() {
                    Some(x) => Ok(ZG1::affine_to_projective(x)),
                    None => Err("Failed to deserialize G1: Affine not available".to_string()),
                }
            })
    }

    fn from_hex(hex: &str) -> Result<Self, String> {
        let bytes = hex::decode(&hex[2..]).unwrap();
        Self::from_bytes(&bytes)
    }

    fn to_bytes(&self) -> [u8; 48] {
        let g1_affine = G1Affine::from(self.proj);
        g1_affine.to_compressed()
    }

    fn add_or_dbl(&self, b: &Self) -> Self {
        Self {
            proj: self.proj + b.proj,
        }
    }
    fn is_inf(&self) -> bool {
        bool::from(self.proj.is_identity())
    }
    fn is_valid(&self) -> bool {
        bool::from(self.proj.is_on_curve())
    }

    fn dbl(&self) -> Self {
        Self {
            proj: self.proj.double(),
        }
    }
    fn add(&self, b: &Self) -> Self {
        Self {
            proj: self.proj + b.proj,
        }
    }

    fn sub(&self, b: &Self) -> Self {
        Self {
            proj: self.proj.sub(&b.proj),
        }
    }

    fn equals(&self, b: &Self) -> bool {
        self.proj.eq(&b.proj)
    }

    fn add_or_dbl_assign(&mut self, b: &Self) {
        self.proj.add_assign(b.proj);
    }

    fn add_assign(&mut self, b: &Self) {
        self.proj.add_assign(b.proj);
    }

    fn dbl_assign(&mut self) {
        self.proj = self.proj.double();
    }

    fn zero() -> Self {
        Self {
            proj: G1Projective {
                x: Fp([
                    8505329371266088957,
                    17002214543764226050,
                    6865905132761471162,
                    8632934651105793861,
                    6631298214892334189,
                    1582556514881692819,
                ]),
                y: Fp([
                    8505329371266088957,
                    17002214543764226050,
                    6865905132761471162,
                    8632934651105793861,
                    6631298214892334189,
                    1582556514881692819,
                ]),
                z: Fp([0, 0, 0, 0, 0, 0]),
            },
        }
    }
}

impl G1Mul<ZFr> for ZG1 {
    fn mul(&self, b: &ZFr) -> Self {
        Self {
            proj: self.proj.mul(b.fr),
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct ZG1Affine(pub G1Affine);
impl G1AffineTrait<ZG1, ZFp> for ZG1Affine {
    fn into_affine(g1: &ZG1) -> Self {
        Self(g1.proj.into())
    }

    fn into_affines(g1: &[ZG1]) -> Vec<Self> {
        let points =
            unsafe { core::slice::from_raw_parts(g1.as_ptr() as *const G1Projective, g1.len()) };
        let mut g1_affine_batch: Vec<G1Affine> = vec![G1Affine::default(); points.len()];
        G1Projective::batch_normalize(points, &mut g1_affine_batch);
        unsafe { core::mem::transmute(g1_affine_batch) }
    }

    fn into_affines_loc(out: &mut [Self], g1: &[ZG1]) {
        out.copy_from_slice(&Self::into_affines(g1));
    }

    fn to_proj(&self) -> ZG1 {
        ZG1 {
            proj: self.0.into(),
        }
    }

    fn x(&self) -> &ZFp {
        unsafe { core::mem::transmute(&self.0.x) }
    }

    fn y(&self) -> &ZFp {
        unsafe { core::mem::transmute(&self.0.y) }
    }

    fn is_infinity(&self) -> bool {
        bool::from(self.0.infinity)
    }

    fn is_zero(&self) -> bool {
        bool::from(self.0.infinity)
        // FIXME: definetly wrong
    }

    fn zero() -> Self {
        Self(G1Affine {
            x: ZFp::zero().0,
            y: ZFp::zero().0,
            infinity: Choice::from(1),
        })
    }

    fn x_mut(&mut self) -> &mut ZFp {
        unsafe { core::mem::transmute(&mut self.0.x) }
    }

    fn y_mut(&mut self) -> &mut ZFp {
        unsafe { core::mem::transmute(&mut self.0.y) }
    }

    // fn double(&mut self) {
    //     self.0.add(&self.0);
    // }

    // fn add(&mut self, b: &Self) {
    //     self.0 += b.0;
    // }
}

pub struct ZG1ProjAddAffine;
impl G1ProjAddAffine<ZG1, ZFp, ZG1Affine> for ZG1ProjAddAffine {
    fn add_assign_affine(proj: &mut ZG1, aff: &ZG1Affine) {
        proj.proj += aff.0;
    }

    fn add_or_double_assign_affine(proj: &mut ZG1, aff: &ZG1Affine) {
        proj.proj += aff.0;
    }
}

impl G1GetFp<ZFp> for ZG1 {
    fn x(&self) -> &ZFp {
        unsafe {
            // Transmute safe due to repr(C) on ZFp
            core::mem::transmute(&self.proj.x)
        }
    }

    fn y(&self) -> &ZFp {
        unsafe {
            // Transmute safe due to repr(C) on ZFp
            core::mem::transmute(&self.proj.y)
        }
    }

    fn z(&self) -> &ZFp {
        unsafe {
            // Transmute safe due to repr(C) on ZFp
            core::mem::transmute(&self.proj.z)
        }
    }

    fn x_mut(&mut self) -> &mut ZFp {
        unsafe {
            // Transmute safe due to repr(C) on ZFp
            core::mem::transmute(&mut self.proj.x)
        }
    }

    fn y_mut(&mut self) -> &mut ZFp {
        unsafe {
            // Transmute safe due to repr(C) on ZFp
            core::mem::transmute(&mut self.proj.y)
        }
    }

    fn z_mut(&mut self) -> &mut ZFp {
        unsafe {
            // Transmute safe due to repr(C) on ZFp
            core::mem::transmute(&mut self.proj.z)
        }
    }
}

impl G1LinComb<ZFr, ZFp, ZG1Affine> for ZG1 {
    fn g1_lincomb(
        points: &[Self],
        scalars: &[ZFr],
        len: usize,
        precomputation: Option<&PrecomputationTable<ZFr, Self, ZFp, ZG1Affine>>,
    ) -> Self {
        let mut out = ZG1::default();
        g1_linear_combination(&mut out, points, scalars, len, precomputation);
        out
    }
}

impl PairingVerify<ZG1, ZG2> for ZG1 {
    fn verify(a1: &ZG1, a2: &ZG2, b1: &ZG1, b2: &ZG2) -> bool {
        pairings_verify(a1, a2, b1, b2)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct ZG2 {
    pub proj: G2Projective,
}

impl ZG2 {
    pub const fn from_blst_p2(p2: blst::blst_p2) -> Self {
        Self {
            proj: blst_p2_into_pc_g2projective(&p2),
        }
    }
    pub const fn from_g2_projective(proj: G2Projective) -> Self {
        Self { proj }
    }
    pub const fn to_blst_p2(&self) -> blst::blst_p2 {
        pc_g2projective_into_blst_p2(self.proj)
    }
}

impl G2 for ZG2 {
    fn generator() -> Self {
        G2_GENERATOR
    }

    fn negative_generator() -> Self {
        G2_NEGATIVE_GENERATOR
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
                let affine = G2Affine::from_compressed(bytes).unwrap();
                Ok(ZG2::from_g2_projective(G2Projective::from(affine)))
            })
    }

    fn to_bytes(&self) -> [u8; 96] {
        let g2_affine = G2Affine::from(self.proj);
        g2_affine.to_compressed()
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        Self {
            proj: self.proj + b.proj,
        }
    }

    fn dbl(&self) -> Self {
        Self {
            proj: self.proj.double(),
        }
    }

    fn sub(&self, b: &Self) -> Self {
        Self {
            proj: self.proj - b.proj,
        }
    }

    fn equals(&self, b: &Self) -> bool {
        self.proj.eq(&b.proj)
    }
}

impl G2Mul<ZFr> for ZG2 {
    fn mul(&self, b: &ZFr) -> Self {
        Self {
            proj: self.proj.mul(b.fr),
        }
    }
}

impl Default for ZFFTSettings {
    fn default() -> Self {
        Self::new(0).unwrap()
    }
}

pub fn fft_g1_fast(
    ret: &mut [ZG1],
    data: &[ZG1],
    stride: usize,
    roots: &[ZFr],
    roots_stride: usize,
) {
    let half = ret.len() / 2;
    if half > 0 {
        #[cfg(feature = "parallel")]
        {
            let (lo, hi) = ret.split_at_mut(half);
            rayon::join(
                || fft_g1_fast(lo, data, stride * 2, roots, roots_stride * 2),
                || fft_g1_fast(hi, &data[stride..], stride * 2, roots, roots_stride * 2),
            );
        }

        #[cfg(not(feature = "parallel"))]
        {
            fft_g1_fast(&mut ret[..half], data, stride * 2, roots, roots_stride * 2);
            fft_g1_fast(
                &mut ret[half..],
                &data[stride..],
                stride * 2,
                roots,
                roots_stride * 2,
            );
        }

        for i in 0..half {
            let y_times_root = ret[i + half].mul(&roots[i * roots_stride]);
            ret[i + half] = ret[i].sub(&y_times_root);
            ret[i] = ret[i].add_or_dbl(&y_times_root);
        }
    } else {
        ret[0] = data[0];
    }
}

impl FFTSettings<ZFr> for ZFFTSettings {
    fn new(scale: usize) -> Result<Self, String> {
        if scale >= SCALE2_ROOT_OF_UNITY.len() {
            return Err(String::from(
                "Scale is expected to be within root of unity matrix row size",
            ));
        }

        // max_width = 2 ^ max_scale
        let max_width: usize = 1 << scale;
        let root_of_unity = ZFr::from_u64_arr(&SCALE2_ROOT_OF_UNITY[scale]);

        // create max_width of roots & store them reversed as well
        let roots_of_unity = expand_root_of_unity(&root_of_unity, max_width)?;

        let mut brp_roots_of_unity = roots_of_unity.clone();
        brp_roots_of_unity.pop();
        reverse_bit_order(&mut brp_roots_of_unity)?;

        let mut reverse_roots_of_unity = roots_of_unity.clone();
        reverse_roots_of_unity.reverse();

        Ok(Self {
            max_width,
            root_of_unity,
            reverse_roots_of_unity,
            roots_of_unity,
            brp_roots_of_unity,
        })
    }

    fn get_max_width(&self) -> usize {
        self.max_width
    }

    fn get_reverse_roots_of_unity_at(&self, i: usize) -> ZFr {
        self.reverse_roots_of_unity[i]
    }

    fn get_reversed_roots_of_unity(&self) -> &[ZFr] {
        &self.reverse_roots_of_unity
    }

    fn get_roots_of_unity_at(&self, i: usize) -> ZFr {
        self.roots_of_unity[i]
    }

    fn get_roots_of_unity(&self) -> &[ZFr] {
        &self.roots_of_unity
    }

    fn get_brp_roots_of_unity(&self) -> &[ZFr] {
        &self.brp_roots_of_unity
    }

    fn get_brp_roots_of_unity_at(&self, i: usize) -> ZFr {
        self.brp_roots_of_unity[i]
    }
}

fn toeplitz_part_1(
    field_elements_per_ext_blob: usize,
    output: &mut [ZG1],
    x: &[ZG1],
    s: &ZFFTSettings,
) -> Result<(), String> {
    let n = x.len();
    let n2 = n * 2;
    let mut x_ext = vec![ZG1::identity(); n2];

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

impl KZGSettings<ZFr, ZG1, ZG2, ZFFTSettings, PolyData, ZFp, ZG1Affine> for ZKZGSettings {
    fn new(
        g1_monomial: &[ZG1],
        g1_lagrange_brp: &[ZG1],
        g2_monomial: &[ZG2],
        fft_settings: &ZFFTSettings,
        cell_size: usize,
    ) -> Result<Self, String> {
        if g1_monomial.len() != g1_lagrange_brp.len() {
            return Err("G1 point length mismatch".to_string());
        }

        let field_elements_per_blob = g1_monomial.len();
        let field_elements_per_ext_blob = field_elements_per_blob * 2;

        let n = field_elements_per_ext_blob / 2;
        let k = n / cell_size;
        let k2 = 2 * k;

        let mut points = vec![ZG1::default(); k2];
        let mut x = vec![ZG1::default(); k];
        let mut x_ext_fft_columns = vec![vec![ZG1::default(); cell_size]; k2];

        for offset in 0..cell_size {
            let start = n - cell_size - 1 - offset;
            for (i, p) in x.iter_mut().enumerate().take(k - 1) {
                let j = start - i * cell_size;
                *p = g1_monomial[j];
            }
            x[k - 1] = ZG1::identity();

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
            precomputation: precompute(g1_lagrange_brp).ok().flatten().map(Arc::new),
            cell_size,
        })
    }

    fn commit_to_poly(&self, p: &PolyData) -> Result<ZG1, String> {
        if p.coeffs.len() > self.g1_values_lagrange_brp.len() {
            return Err(String::from("Polynomial is longer than secret g1"));
        }

        let mut out = ZG1::default();
        g1_linear_combination(
            &mut out,
            &self.g1_values_lagrange_brp,
            &p.coeffs,
            p.coeffs.len(),
            None,
        );

        Ok(out)
    }

    fn compute_proof_single(&self, p: &PolyData, x: &ZFr) -> Result<ZG1, String> {
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
    }

    fn check_proof_single(&self, com: &ZG1, proof: &ZG1, x: &ZFr, y: &ZFr) -> Result<bool, String> {
        let x_g2 = G2_GENERATOR.mul(x);
        let s_minus_x: ZG2 = self.g2_values_monomial[1].sub(&x_g2);
        let y_g1 = G1_GENERATOR.mul(y);
        let commitment_minus_y: ZG1 = com.sub(&y_g1);

        Ok(pairings_verify(
            &commitment_minus_y,
            &G2_GENERATOR,
            proof,
            &s_minus_x,
        ))
    }

    fn compute_proof_multi(&self, p: &PolyData, x: &ZFr, n: usize) -> Result<ZG1, String> {
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
            divisor.coeffs.push(ZFr { fr: Scalar::zero() });
        }

        // x^n
        divisor.coeffs.push(ZFr { fr: Scalar::one() });

        let mut new_polina = p.clone();

        // Calculate q = p / (x^n - x0^n)
        // let q = p.div(&divisor).unwrap();
        let q = new_polina.div(&divisor)?;
        let ret = self.commit_to_poly(&q)?;
        Ok(ret)
    }

    fn check_proof_multi(
        &self,
        com: &ZG1,
        proof: &ZG1,
        x: &ZFr,
        ys: &[ZFr],
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

        let xn2 = G2_GENERATOR.mul(&x_pow);

        // [s^n - x^n]_2
        let xn_minus_yn = self.g2_values_monomial[n].sub(&xn2);

        // [interpolation_polynomial(s)]_1
        let is1 = self.commit_to_poly(&interp).unwrap();

        // [commitment - interpolation_polynomial(s)]_1 = [commit]_1 - [interpolation_polynomial(s)]_1
        let commit_minus_interp = com.sub(&is1);
        let ret = pairings_verify(&commit_minus_interp, &G2_GENERATOR, proof, &xn_minus_yn);

        Ok(ret)
    }

    fn get_roots_of_unity_at(&self, i: usize) -> ZFr {
        self.fs.get_roots_of_unity_at(i)
    }

    fn get_fft_settings(&self) -> &ZFFTSettings {
        &self.fs
    }

    fn get_precomputation(&self) -> Option<&PrecomputationTable<ZFr, ZG1, ZFp, ZG1Affine>> {
        self.precomputation.as_ref().map(|v| v.as_ref())
    }

    fn get_g1_monomial(&self) -> &[ZG1] {
        &self.g1_values_monomial
    }

    fn get_g1_lagrange_brp(&self) -> &[ZG1] {
        &self.g1_values_lagrange_brp
    }

    fn get_g2_monomial(&self) -> &[ZG2] {
        &self.g2_values_monomial
    }

    fn get_x_ext_fft_column(&self, index: usize) -> &[ZG1] {
        &self.x_ext_fft_columns[index]
    }

    fn get_cell_size(&self) -> usize {
        self.cell_size
    }
}

impl<'a> TryFrom<&'a CKZGSettings> for ZKZGSettings {
    type Error = String;

    fn try_from(c_settings: &'a CKZGSettings) -> Result<Self, Self::Error> {
        Ok(ZKZGSettings {
            fs: fft_settings_to_rust(c_settings)?,
            g1_values_monomial: unsafe {
                core::slice::from_raw_parts(
                    c_settings.g1_values_monomial,
                    eth::FIELD_ELEMENTS_PER_BLOB,
                )
            }
            .iter()
            .map(|r| ZG1::from_blst_p1(*r))
            .collect::<Vec<_>>(),
            g1_values_lagrange_brp: unsafe {
                core::slice::from_raw_parts(
                    c_settings.g1_values_lagrange_brp,
                    eth::FIELD_ELEMENTS_PER_BLOB,
                )
            }
            .iter()
            .map(|r| ZG1::from_blst_p1(*r))
            .collect::<Vec<_>>(),
            g2_values_monomial: unsafe {
                core::slice::from_raw_parts(
                    c_settings.g2_values_monomial,
                    eth::TRUSTED_SETUP_NUM_G2_POINTS,
                )
            }
            .iter()
            .map(|r| ZG2::from_blst_p2(*r))
            .collect::<Vec<_>>(),
            x_ext_fft_columns: unsafe {
                core::slice::from_raw_parts(
                    c_settings.x_ext_fft_columns,
                    2 * ((eth::FIELD_ELEMENTS_PER_EXT_BLOB / 2) / eth::FIELD_ELEMENTS_PER_CELL),
                )
            }
            .iter()
            .map(|it| {
                unsafe { core::slice::from_raw_parts(*it, eth::FIELD_ELEMENTS_PER_CELL) }
                    .iter()
                    .map(|it| ZG1::from_blst_p1(*it))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
            precomputation: unsafe { PRECOMPUTATION_TABLES.get_precomputation(c_settings) },
            cell_size: eth::FIELD_ELEMENTS_PER_CELL,
        })
    }
}

use crate::consts::{
    G1_GENERATOR, G1_IDENTITY, G1_NEGATIVE_GENERATOR, G2_GENERATOR, G2_NEGATIVE_GENERATOR,
    SCALE2_ROOT_OF_UNITY,
};
use crate::fft_g1::g1_linear_combination;
use crate::kzg_proofs::{
    eval_poly, expand_root_of_unity, pairings_verify, FFTSettings as LFFTSettings,
    KZGSettings as LKZGSettings,
};
use crate::poly::{poly_fast_div, poly_inverse, poly_long_div, poly_mul_direct, poly_mul_fft};
use crate::recover::{scale_poly, unscale_poly};
use crate::utils::{
    blst_fp_into_pc_fq, blst_fr_into_pc_fr, blst_p1_into_pc_g1projective,
    blst_p2_into_pc_g2projective, pc_fr_into_blst_fr, pc_g1projective_into_blst_p1,
    pc_g2projective_into_blst_p2, PolyData,
};
use ark_bls12_381::{g1, g2, Fr, G1Affine, G2Affine};
use ark_ec::{models::short_weierstrass::Projective, AffineRepr, Group};
use ark_ec::{CurveConfig, CurveGroup};
use ark_ff::{biginteger::BigInteger256, BigInteger, Field};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::{One, Zero};

#[cfg(feature = "rand")]
use ark_std::UniformRand;

use blst::{blst_fp, blst_fr, blst_p1};
use kzg::common_utils::reverse_bit_order;
use kzg::eip_4844::{BYTES_PER_FIELD_ELEMENT, BYTES_PER_G1, BYTES_PER_G2};
use kzg::msm::precompute::{precompute, PrecomputationTable};
use kzg::{
    FFTFr, FFTSettings, FFTSettingsPoly, Fr as KzgFr, G1Affine as G1AffineTrait, G1Fp, G1GetFp,
    G1LinComb, G1Mul, G1ProjAddAffine, G2Mul, KZGSettings, PairingVerify, Poly, Scalar256, G1, G2,
};
use std::ops::{AddAssign, Mul, Neg, Sub};

fn bytes_be_to_uint64(inp: &[u8]) -> u64 {
    u64::from_be_bytes(inp.try_into().expect("Input wasn't 8 elements..."))
}

const BLS12_381_MOD_256: [u64; 4] = [
    0xffffffff00000001,
    0x53bda402fffe5bfe,
    0x3339d80809a1d805,
    0x73eda753299d7d48,
];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct ArkFr {
    pub fr: Fr,
}

impl ArkFr {
    pub fn from_blst_fr(fr: blst_fr) -> Self {
        Self {
            fr: blst_fr_into_pc_fr(fr),
        }
    }

    pub fn to_blst_fr(&self) -> blst_fr {
        pc_fr_into_blst_fr(self.fr)
    }
}

fn bigint_check_mod_256(a: &[u64; 4]) -> bool {
    let (_, overflow) = a[0].overflowing_sub(BLS12_381_MOD_256[0]);
    let (_, overflow) = a[1].overflowing_sub(BLS12_381_MOD_256[1] + overflow as u64);
    let (_, overflow) = a[2].overflowing_sub(BLS12_381_MOD_256[2] + overflow as u64);
    let (_, overflow) = a[3].overflowing_sub(BLS12_381_MOD_256[3] + overflow as u64);
    overflow
}

impl KzgFr for ArkFr {
    fn null() -> Self {
        Self {
            fr: Fr::new_unchecked(BigInteger256::new([u64::MAX; 4])),
        }
    }

    fn zero() -> Self {
        // Self::from_u64(0)
        Self { fr: Fr::zero() }
    }

    fn one() -> Self {
        let one = Fr::one();
        // assert_eq!(one.0.0, [0, 1, 1, 1], "must be eq");
        Self { fr: one }
        // Self::from_u64(1)
    }

    #[cfg(feature = "rand")]
    fn rand() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            fr: Fr::rand(&mut rng),
        }
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
                let storage: [u64; 4] = [
                    bytes_be_to_uint64(&bytes[24..32]),
                    bytes_be_to_uint64(&bytes[16..24]),
                    bytes_be_to_uint64(&bytes[8..16]),
                    bytes_be_to_uint64(&bytes[0..8]),
                ];
                let big_int = BigInteger256::new(storage);
                if !big_int.is_zero() && !bigint_check_mod_256(&big_int.0) {
                    return Err("Invalid scalar".to_string());
                }
                Ok(Self {
                    fr: Fr::new(big_int),
                })
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
                let storage: [u64; 4] = [
                    bytes_be_to_uint64(&bytes[24..32]),
                    bytes_be_to_uint64(&bytes[16..24]),
                    bytes_be_to_uint64(&bytes[8..16]),
                    bytes_be_to_uint64(&bytes[0..8]),
                ];
                let big_int = BigInteger256::new(storage);
                Self {
                    fr: Fr::new(big_int),
                }
            })
    }

    fn from_hex(hex: &str) -> Result<Self, String> {
        let bytes = hex::decode(&hex[2..]).unwrap();
        Self::from_bytes(&bytes)
    }

    fn from_u64_arr(u: &[u64; 4]) -> Self {
        Self {
            fr: Fr::new(BigInteger256::new(*u)),
        }
    }

    fn from_u64(val: u64) -> Self {
        Self::from_u64_arr(&[val, 0, 0, 0])
    }

    fn to_bytes(&self) -> [u8; 32] {
        let big_int_256: BigInteger256 = Fr::into(self.fr);
        <[u8; 32]>::try_from(big_int_256.to_bytes_be()).unwrap()
    }

    fn to_u64_arr(&self) -> [u64; 4] {
        let b: BigInteger256 = Fr::into(self.fr);
        b.0
    }

    fn is_one(&self) -> bool {
        self.fr.is_one()
    }

    fn is_zero(&self) -> bool {
        self.fr.is_zero()
    }

    fn is_null(&self) -> bool {
        self.equals(&ArkFr::null())
    }

    fn sqr(&self) -> Self {
        Self {
            fr: self.fr.square(),
        }
    }

    fn mul(&self, b: &Self) -> Self {
        Self { fr: self.fr * b.fr }
    }

    fn add(&self, b: &Self) -> Self {
        Self { fr: self.fr + b.fr }
    }

    fn sub(&self, b: &Self) -> Self {
        Self { fr: self.fr - b.fr }
    }

    fn eucl_inverse(&self) -> Self {
        // Inverse and eucl inverse work the same way
        Self {
            fr: self.fr.inverse().unwrap(),
        }
    }

    fn negate(&self) -> Self {
        Self { fr: self.fr.neg() }
    }

    fn inverse(&self) -> Self {
        Self {
            fr: self.fr.inverse().unwrap(),
        }
    }

    fn pow(&self, n: usize) -> Self {
        Self {
            fr: self.fr.pow([n as u64]),
        }
    }

    fn div(&self, b: &Self) -> Result<Self, String> {
        let div = self.fr / b.fr;
        if div.0 .0.is_empty() {
            Ok(Self { fr: Fr::zero() })
        } else {
            Ok(Self { fr: div })
        }
    }

    fn equals(&self, b: &Self) -> bool {
        self.fr == b.fr
    }

    fn to_scalar(&self) -> Scalar256 {
        Scalar256::from_u64(BigInteger256::from(self.fr).0)
    }
}

#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct ArkG1(pub Projective<g1::Config>);

impl ArkG1 {
    pub const fn from_blst_p1(p1: blst_p1) -> Self {
        Self(blst_p1_into_pc_g1projective(&p1))
    }

    pub const fn to_blst_p1(&self) -> blst_p1 {
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
        Self(Projective::rand(&mut rng))
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
                let affine = G1Affine::deserialize_compressed(bytes.as_slice());
                match affine {
                    Err(x) => Err("Failed to deserialize G1: ".to_owned() + &(x.to_string())),
                    Ok(x) => Ok(Self(x.into_group())),
                }
            })
    }

    fn from_hex(hex: &str) -> Result<Self, String> {
        let bytes = hex::decode(&hex[2..]).unwrap();
        Self::from_bytes(&bytes)
    }

    fn to_bytes(&self) -> [u8; 48] {
        let mut buff = [0u8; BYTES_PER_G1];
        self.0.serialize_compressed(&mut &mut buff[..]).unwrap();
        buff
    }

    fn add_or_dbl(&self, b: &Self) -> Self {
        Self(self.0 + b.0)
    }

    fn is_inf(&self) -> bool {
        let temp = &self.0;
        temp.z.is_zero()
    }

    fn is_valid(&self) -> bool {
        true
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

    const ZERO: ArkG1 = ArkG1::from_blst_p1(blst_p1 {
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
        Self(self.0.mul(b.fr))
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
pub struct ArkG2(pub Projective<g2::Config>);

impl ArkG2 {
    pub const fn from_blst_p2(p2: blst::blst_p2) -> Self {
        Self(blst_p2_into_pc_g2projective(&p2))
    }

    pub const fn to_blst_p2(&self) -> blst::blst_p2 {
        pc_g2projective_into_blst_p2(self.0)
    }
}

impl G2 for ArkG2 {
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
                let affine = G2Affine::deserialize_compressed(bytes.as_slice());
                match affine {
                    Err(x) => Err("Failed to deserialize G2: ".to_owned() + &(x.to_string())),
                    Ok(x) => Ok(Self(x.into_group())),
                }
            })
    }

    fn to_bytes(&self) -> [u8; 96] {
        let mut buff = [0u8; BYTES_PER_G2];
        self.0.serialize_compressed(&mut &mut buff[..]).unwrap();
        buff
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
        Self(self.0.mul(&b.fr))
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
        eval_poly(self, x)
    }

    fn scale(&mut self) {
        scale_poly(self);
    }

    fn unscale(&mut self) {
        unscale_poly(self);
    }

    fn inverse(&mut self, new_len: usize) -> Result<Self, String> {
        poly_inverse(self, new_len)
    }

    fn div(&mut self, x: &Self) -> Result<Self, String> {
        if x.len() >= self.len() || x.len() < 128 {
            poly_long_div(self, x)
        } else {
            poly_fast_div(self, x)
        }
    }

    fn long_div(&mut self, x: &Self) -> Result<Self, String> {
        poly_long_div(self, x)
    }

    fn fast_div(&mut self, x: &Self) -> Result<Self, String> {
        poly_fast_div(self, x)
    }

    fn mul_direct(&mut self, x: &Self, len: usize) -> Result<Self, String> {
        poly_mul_direct(self, x, len)
    }
}

impl FFTSettingsPoly<ArkFr, PolyData, LFFTSettings> for LFFTSettings {
    fn poly_mul_fft(
        a: &PolyData,
        x: &PolyData,
        len: usize,
        fs: Option<&LFFTSettings>,
    ) -> Result<PolyData, String> {
        poly_mul_fft(a, x, fs, len)
    }
}

impl Default for LFFTSettings {
    fn default() -> Self {
        Self {
            max_width: 0,
            root_of_unity: ArkFr::zero(),
            expanded_roots_of_unity: Vec::new(),
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

        let expanded_roots_of_unity = expand_root_of_unity(&root_of_unity, max_width)?;
        let mut reverse_roots_of_unity = expanded_roots_of_unity.clone();
        reverse_roots_of_unity.reverse();

        let mut roots_of_unity = expanded_roots_of_unity.clone();
        roots_of_unity.pop();
        reverse_bit_order(&mut roots_of_unity)?;

        Ok(LFFTSettings {
            max_width,
            root_of_unity,
            expanded_roots_of_unity,
            reverse_roots_of_unity,
            roots_of_unity,
        })
    }

    fn get_max_width(&self) -> usize {
        self.max_width
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> ArkFr {
        self.expanded_roots_of_unity[i]
    }

    fn get_expanded_roots_of_unity(&self) -> &[ArkFr] {
        &self.expanded_roots_of_unity
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

impl KZGSettings<ArkFr, ArkG1, ArkG2, LFFTSettings, PolyData, ArkFp, ArkG1Affine> for LKZGSettings {
    fn new(
        secret_g1: &[ArkG1],
        secret_g2: &[ArkG2],
        _length: usize,
        fft_settings: &LFFTSettings,
    ) -> Result<LKZGSettings, String> {
        Ok(Self {
            secret_g1: secret_g1.to_vec(),
            secret_g2: secret_g2.to_vec(),
            fs: fft_settings.clone(),
            precomputation: precompute(secret_g1).ok().flatten(),
        })
    }

    fn commit_to_poly(&self, p: &PolyData) -> Result<ArkG1, String> {
        if p.coeffs.len() > self.secret_g1.len() {
            return Err(String::from("Polynomial is longer than secret g1"));
        }

        let mut out = ArkG1::default();
        g1_linear_combination(
            &mut out,
            &self.secret_g1,
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
        let x_g2: ArkG2 = G2_GENERATOR.mul(x);
        let s_minus_x: ArkG2 = self.secret_g2[1].sub(&x_g2);
        let y_g1 = G1_GENERATOR.mul(y);
        let commitment_minus_y: ArkG1 = com.sub(&y_g1);

        Ok(pairings_verify(
            &commitment_minus_y,
            &G2_GENERATOR,
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
            divisor.coeffs.push(ArkFr { fr: Fr::zero() });
        }

        // x^n
        divisor.coeffs.push(ArkFr { fr: Fr::one() });

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

        let xn2 = G2_GENERATOR.mul(&x_pow);

        // [s^n - x^n]_2
        let xn_minus_yn = self.secret_g2[n].sub(&xn2);

        // [interpolation_polynomial(s)]_1
        let is1 = self.commit_to_poly(&interp).unwrap();

        // [commitment - interpolation_polynomial(s)]_1 = [commit]_1 - [interpolation_polynomial(s)]_1
        let commit_minus_interp = com.sub(&is1);

        let ret = pairings_verify(&commit_minus_interp, &G2_GENERATOR, proof, &xn_minus_yn);

        Ok(ret)
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> ArkFr {
        self.fs.get_expanded_roots_of_unity_at(i)
    }

    fn get_roots_of_unity_at(&self, i: usize) -> ArkFr {
        self.fs.get_roots_of_unity_at(i)
    }

    fn get_fft_settings(&self) -> &LFFTSettings {
        &self.fs
    }

    fn get_g1_secret(&self) -> &[ArkG1] {
        &self.secret_g1
    }

    fn get_g2_secret(&self) -> &[ArkG2] {
        &self.secret_g2
    }

    fn get_precomputation(&self) -> Option<&PrecomputationTable<ArkFr, ArkG1, ArkFp, ArkG1Affine>> {
        self.precomputation.as_ref()
    }
}

type ArkFpInt = <ark_bls12_381::g1::Config as CurveConfig>::BaseField;
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

    const ZERO: Self = Self(ArkFpInt::ZERO);
    const ONE: Self = Self(ArkFpInt::ONE);
    const BLS12_381_RX_P: Self = Self(blst_fp_into_pc_fq(&blst_fp {
        l: [
            8505329371266088957,
            17002214543764226050,
            6865905132761471162,
            8632934651105793861,
            6631298214892334189,
            1582556514881692819,
        ],
    }));
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
        let ark_points: &[Projective<g1::Config>] = unsafe { core::mem::transmute(g1) };
        let ark_points = CurveGroup::normalize_batch(ark_points);
        unsafe { core::mem::transmute(ark_points) }
    }

    fn into_affines_loc(out: &mut [Self], g1: &[ArkG1]) {
        out.copy_from_slice(&Self::into_affines(g1));
    }

    fn to_proj(&self) -> ArkG1 {
        ArkG1(self.aff.into_group())
    }

    fn x(&self) -> &ArkFp {
        unsafe { core::mem::transmute(&self.aff.x) }
    }

    fn y(&self) -> &ArkFp {
        unsafe { core::mem::transmute(&self.aff.y) }
    }

    fn is_infinity(&self) -> bool {
        self.aff.infinity
    }

    fn is_zero(&self) -> bool {
        self.aff.is_zero()
    }

    fn zero() -> Self {
        Self {
            aff: G1Affine {
                x: ArkFp::ZERO.0,
                y: ArkFp::ZERO.0,
                infinity: true,
            },
        }
    }

    fn x_mut(&mut self) -> &mut ArkFp {
        unsafe { core::mem::transmute(&mut self.aff.x) }
    }

    fn y_mut(&mut self) -> &mut ArkFp {
        unsafe { core::mem::transmute(&mut self.aff.y) }
    }
}

pub struct ArkG1ProjAddAffine;
impl G1ProjAddAffine<ArkG1, ArkFp, ArkG1Affine> for ArkG1ProjAddAffine {
    fn add_assign_affine(proj: &mut ArkG1, aff: &ArkG1Affine) {
        proj.0 += aff.aff;
    }

    fn add_or_double_assign_affine(proj: &mut ArkG1, aff: &ArkG1Affine) {
        proj.0 += aff.aff;
    }
}

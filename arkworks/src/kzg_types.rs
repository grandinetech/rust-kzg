use crate::consts::{SCALE2_ROOT_OF_UNITY, G1_GENERATOR, G1_IDENTITY, G1_NEGATIVE_GENERATOR, G2_GENERATOR, G2_NEGATIVE_GENERATOR};
use crate::fft_g1::g1_linear_combination;
use crate::kzg_proofs::{
    expand_root_of_unity, FFTSettings as LFFTSettings,
    KZGSettings as LKZGSettings, eval_poly,
};
use crate::utils::{PolyData, blst_p2_into_pc_g2projective};
use ark_ec::scalar_mul::fixed_base::FixedBase;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, CanonicalSerializeHashExt};
use crate::poly::{poly_fast_div, poly_inverse, poly_long_div, poly_mul_direct, poly_mul_fft};
use crate::recover::{scale_poly, unscale_poly};
use crate::utils::PolyData as LPoly;
use ark_poly::Polynomial;
use crate::utils::{
    blst_fr_into_pc_fr, blst_p1_into_pc_g1projective,
};
use crate::kzg_proofs::pairings_verify;
use ark_bls12_381::{g1, g2, Fr, G1Affine, G2Affine};
use ark_ec::models::short_weierstrass::{Projective};
use ark_ec::{AffineRepr, VariableBaseMSM};
use ark_ec::short_weierstrass::SWCurveConfig;
use ark_ff::{PrimeField, BigInt};
use ark_ff::{biginteger::BigInteger256, BigInteger, Field};
use ark_poly::{EvaluationDomain, GeneralEvaluationDomain};
use ark_std::{One, UniformRand, Zero};
use blst::{blst_fr, blst_p1};
use kzg::common_utils::reverse_bit_order;
use kzg::eip_4844::{BYTES_PER_FIELD_ELEMENT, BYTES_PER_G1, BYTES_PER_G2};
use kzg::{FFTSettings, FFTSettingsPoly, Fr as KzgFr, G1Mul, G2Mul, KZGSettings, Poly, G1, G2, FFTFr, PairingVerify};

use std::ops::{MulAssign, Sub, SubAssign, AddAssign};
use ark_ff::FftField;
use std::ops::Neg;
use ark_ec::CurveGroup;
use ark_ec::Group;
use std::ops::Mul;


#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct ArkFr {
    pub fr: Fr,
}

impl ArkFr {
    pub fn from_blst_fr(fr: blst_fr) -> Self {
        Self { fr: blst_fr_into_pc_fr(fr) }
    }
}

impl KzgFr for ArkFr {
    fn null() -> Self {
        ArkFr::from_blst_fr(blst_fr {
            l: [
                14526898868952669296,
                2784871451429007392,
                11493358522590675359,
                7138715389977065193,
            ],
        })
    }

    fn zero() -> Self {
        Self::from_u64(0)
    }

    fn one() -> Self {
        Self::from_u64(1)
    }

    fn rand() -> Self {
        let mut rng = rand::thread_rng();
        Self { fr: Fr::rand(&mut rng) }
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
                let mut le_bytes: [u8; BYTES_PER_FIELD_ELEMENT] = bytes.clone();
                le_bytes.reverse();
                let fr = Fr::deserialize_compressed(le_bytes.as_slice()).unwrap_or_default();
                Ok(Self { fr })
            })
    }

    fn from_hex(hex: &str) -> Result<Self, String> {
        let bytes = hex::decode(&hex[2..]).unwrap();
        Self::from_bytes(&bytes)
    }

    fn from_u64_arr(u: &[u64; 4]) -> Self {
        Self { fr: Fr::new(BigInteger256::new(*u)) }
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
        Self { fr: self.fr.square() }
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
        Self { fr: self.fr.inverse().unwrap() }
    }

    fn negate(&self) -> Self {
        Self { fr: self.fr.neg() }
    }

    fn inverse(&self) -> Self {
        Self { fr: self.fr.inverse().unwrap() }
    }

    fn pow(&self, n: usize) -> Self {
        Self { fr: self.fr.pow([n as u64]) }
    }

    fn div(&self, b: &Self) -> Result<Self, String> {
        let div = self.fr / b.fr;
        if div.0 .0.is_empty() {
            Ok(Self { fr: Fr::zero() } )
        } else {
            Ok(Self { fr: div } )
        }
    }

    fn equals(&self, b: &Self) -> bool {
        self.fr == b.fr
    }
}


#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct ArkG1 {
    pub proj: Projective<g1::Config>
}

impl ArkG1 {
    pub const fn from_blst_p1(p2: blst_p1) -> Self {
        Self {proj: blst_p1_into_pc_g1projective(&p2)}
    }
}

impl From<blst_p1> for ArkG1 {
    fn from(p1: blst_p1) -> Self {
        let proj = blst_p1_into_pc_g1projective(&p1);
        Self {proj}
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

    fn rand() -> Self {
        let mut rng = rand::thread_rng();
        Self { proj: Projective::rand(&mut rng) }
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
                let affine = G1Affine::deserialize_compressed_unchecked(bytes.as_slice());
                match affine {
                    Err(x) => Err("Failed to deserialize G1: ".to_owned() + &(x.to_string())),
                    Ok(x) => Ok(Self { proj: x.into_group() })
                }
            })
    }

    fn from_hex(hex: &str) -> Result<Self, String> {
        let bytes = hex::decode(&hex[2..]).unwrap();
        Self::from_bytes(&bytes)
    }

    fn to_bytes(&self) -> [u8; 48] {
        let mut buff: [u8; 48] = [0; 48];
        self.proj.serialize_compressed(&mut &mut buff[..]).unwrap();
        buff
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        let res = &self.proj
            + &b.proj;
        Self {proj: res}
    }

    fn is_inf(&self) -> bool {
        let temp = &self.proj;
        temp.z.is_zero()
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn dbl(&self) -> Self {
        Self { proj: self.proj.double() }
    }

    fn add(&self, b: &Self) -> Self {
        Self { proj: &self.proj + &b.proj }
    }

    fn sub(&self, b: &Self) -> Self {
        Self { proj: self.proj.sub(&b.proj)}
    }

    fn equals(&self, b: &Self) -> bool {
        self.proj.eq(&b.proj)
    }
}

impl G1Mul<ArkFr> for ArkG1 {
    fn mul(&self, b: &ArkFr) -> Self {
        Self { proj: self.proj.mul(b.fr) }
    }

    fn g1_lincomb(points: &[Self], scalars: &[ArkFr], len: usize) -> Self {
        let mut out = Self::default();
        g1_linear_combination(&mut out, points, scalars, len);
        out
    }
}

impl PairingVerify<ArkG1, ArkG2> for ArkG1 {
    fn verify(a1: &ArkG1, a2: &ArkG2, b1: &ArkG1, b2: &ArkG2) -> bool {
        pairings_verify(a1, a2, b1, b2)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct ArkG2 {
    pub proj: Projective<g2::Config>
}

impl ArkG2 {
    pub const fn from_blst_p2(p2: blst::blst_p2) -> Self {
        Self {proj: blst_p2_into_pc_g2projective(&p2)}
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
                    Ok(x) => Ok(Self { proj: x.into_group() })
                }
            })
    }

    fn to_bytes(&self) -> [u8; 96] {
        let mut buff: [u8; 96] = [0; 96];
        self.proj.serialize_compressed(&mut &mut buff[..]).unwrap();
        buff
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        Self { proj: self.proj + b.proj }
    }

    fn dbl(&self) -> Self {
        Self { proj: self.proj.double() }
    }

    fn sub(&self, b: &Self) -> Self {
        Self { proj: &self.proj - b.proj }
    }

    fn equals(&self, b: &Self) -> bool {
        self.proj.eq(&b.proj)
    }
}

impl G2Mul<ArkFr> for ArkG2 {
    fn mul(&self, b: &ArkFr) -> Self {
        // FIXME: Is this right?
        Self { proj: self.proj.mul(b.fr) }
    }
}

impl Poly<ArkFr> for LPoly {
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

impl FFTSettingsPoly<ArkFr, LPoly, LFFTSettings> for LFFTSettings {
    fn poly_mul_fft(
        a: &LPoly,
        x: &LPoly,
        len: usize,
        fs: Option<&LFFTSettings>,
    ) -> Result<LPoly, String> {
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

impl KZGSettings<ArkFr, ArkG1, ArkG2, LFFTSettings, LPoly> for LKZGSettings {
    fn new(
        secret_g1: &[ArkG1],
        secret_g2: &[ArkG2],
        length: usize,
        fft_settings: &LFFTSettings,
    ) -> Result<LKZGSettings, String> {
        Ok(Self {
            secret_g1: secret_g1.to_vec(),
            secret_g2: secret_g2.to_vec(),
            fs: fft_settings.clone(),
        })
    }

    fn commit_to_poly(&self, p: &LPoly) -> Result<ArkG1, String> {
        if p.coeffs.len() > self.secret_g1.len() {
            return Err(String::from("Polynomial is longer than secret g1"));
        }

        let mut out = ArkG1::default();
        g1_linear_combination(&mut out, &self.secret_g1, &p.coeffs, p.coeffs.len());

        Ok(out)
    }

    fn compute_proof_single(&self, p: &LPoly, x: &ArkFr) -> Result<ArkG1, String> {
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

        let q = LPoly { coeffs: out_coeffs };
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

    fn compute_proof_multi(&self, p: &LPoly, x: &ArkFr, n: usize) -> Result<ArkG1, String> {
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
            divisor.coeffs.push( ArkFr { fr: Fr::zero() } );
        }

        // x^n
        divisor.coeffs.push( ArkFr { fr: Fr::one() } );

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
}

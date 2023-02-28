use crate::fft::SCALE2_ROOT_OF_UNITY;
use crate::fft_g1::{G1_GENERATOR, G1_IDENTITY, G1_NEGATIVE_GENERATOR};
use crate::kzg_proofs::{
    check_proof_multi as check_multi, check_proof_single as check_single, commit_to_poly as commit,
    compute_proof_multi as compute_multi, compute_proof_single as compute_single, default_kzg,
    eval_poly, expand_root_of_unity, new_kzg_settings, FFTSettings as LFFTSettings,
    KZGSettings as LKZGSettings, G2_GENERATOR, G2_NEGATIVE_GENERATOR,
};
use crate::poly::{poly_fast_div, poly_inverse, poly_long_div, poly_mul_direct, poly_mul_fft};
use crate::recover::{scale_poly, unscale_poly};
use crate::utils::PolyData as LPoly;
use crate::utils::{
    blst_fr_into_pc_fr, blst_p1_into_pc_g1projective, blst_p2_into_pc_g2projective,
    pc_fr_into_blst_fr, pc_g1projective_into_blst_p1, pc_g2projective_into_blst_p2,
};
use ark_bls12_381::Fr as ArkFr;
use ark_ec::models::short_weierstrass_jacobian::GroupProjective;
use ark_ec::{AffineCurve, ProjectiveCurve};
use ark_ff::{biginteger::BigInteger256, Field, PrimeField};
use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};
use ark_std::{One, UniformRand, Zero};
use blst::{blst_fr, blst_p1};
use kzg::{FFTSettings, FFTSettingsPoly, Fr, G1Mul, G2Mul, KZGSettings, Poly, G1, G2};
use std::ops::MulAssign;
use std::ops::Neg;

#[derive(Debug, PartialEq, Eq)]
pub struct ArkG1(pub blst::blst_p1);

impl Clone for ArkG1 {
    fn clone(&self) -> Self {
        ArkG1(self.0)
    }
}

impl G1 for ArkG1 {
    fn default() -> Self {
        Self(blst_p1::default())
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        let temp = blst_p1_into_pc_g1projective(&self.0).unwrap()
            + blst_p1_into_pc_g1projective(&b.0).unwrap();
        let ret = pc_g1projective_into_blst_p1(temp).unwrap();
        self.0 = ret.0;
        ret
    }

    fn equals(&self, b: &Self) -> bool {
        self.0.eq(&b.0)
    }

    fn rand() -> Self {
        let mut rng = rand::thread_rng();
        pc_g1projective_into_blst_p1(GroupProjective::rand(&mut rng)).unwrap()
    }

    fn identity() -> Self {
        G1_IDENTITY
    }

    fn generator() -> Self {
        ArkG1(G1_GENERATOR)
    }

    fn negative_generator() -> Self {
        ArkG1(G1_NEGATIVE_GENERATOR)
    }

    fn is_inf(&self) -> bool {
        let temp = blst_p1_into_pc_g1projective(&self.0).unwrap();
        temp.z.is_zero()
    }

    fn dbl(&self) -> Self {
        let temp = blst_p1_into_pc_g1projective(&self.0).unwrap();
        pc_g1projective_into_blst_p1(temp.double()).unwrap()
    }

    fn sub(&self, b: &Self) -> Self {
        pc_g1projective_into_blst_p1(
            blst_p1_into_pc_g1projective(&self.0).unwrap()
                - blst_p1_into_pc_g1projective(&b.0).unwrap(),
        )
        .unwrap()
    }
}

impl G1Mul<FsFr> for ArkG1 {
    fn mul(&self, b: &FsFr) -> Self {
        let a = blst_p1_into_pc_g1projective(&self.0).unwrap().into_affine();
        let b = blst_fr_into_pc_fr(b);
        pc_g1projective_into_blst_p1(a.mul(b)).unwrap()
    }
}

impl Copy for ArkG1 {}

#[derive(Debug)]
pub struct ArkG2(pub blst::blst_p2);

impl Clone for ArkG2 {
    fn clone(&self) -> Self {
        ArkG2(self.0)
    }
}

impl G2 for ArkG2 {
    fn default() -> Self {
        todo!()
    }

    fn generator() -> Self {
        G2_GENERATOR
    }

    fn negative_generator() -> Self {
        G2_NEGATIVE_GENERATOR
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        let temp =
            blst_p2_into_pc_g2projective(self).unwrap() + blst_p2_into_pc_g2projective(b).unwrap();
        let ret = pc_g2projective_into_blst_p2(temp).unwrap();
        self.0 = ret.0;
        ret
    }

    fn dbl(&self) -> Self {
        let temp = blst_p2_into_pc_g2projective(self).unwrap();
        pc_g2projective_into_blst_p2(temp.double()).unwrap()
    }

    fn sub(&self, b: &Self) -> Self {
        pc_g2projective_into_blst_p2(
            blst_p2_into_pc_g2projective(self).unwrap() - blst_p2_into_pc_g2projective(b).unwrap(),
        )
        .unwrap()
    }

    fn equals(&self, b: &Self) -> bool {
        self.0.eq(&b.0)
    }
}

impl G2Mul<FsFr> for ArkG2 {
    fn mul(&self, b: &FsFr) -> Self {
        let mut a = blst_p2_into_pc_g2projective(self).unwrap();
        let b = blst_fr_into_pc_fr(b);
        a.mul_assign(b);
        pc_g2projective_into_blst_p2(a).unwrap()
    }
}

#[derive(Debug)]
pub struct FsFr(pub blst::blst_fr);

impl Fr for FsFr {
    fn default() -> Self {
        Self(blst_fr::default())
    }

    fn zero() -> Self {
        Self::from_u64(0)
    }

    fn null() -> Self {
        FsFr(blst_fr {
            l: [
                14526898868952669296,
                2784871451429007392,
                11493358522590675359,
                7138715389977065193,
            ],
        })
    }

    fn one() -> Self {
        Self::from_u64(1)
    }

    fn rand() -> Self {
        let mut rng = rand::thread_rng();
        pc_fr_into_blst_fr(ArkFr::rand(&mut rng))
    }

    fn from_u64_arr(u: &[u64; 4]) -> Self {
        let b = ArkFr::from_repr(BigInteger256::new(*u));
        match b {
            None => FsFr(blst_fr { l: [0, 0, 0, 0] }),
            Some(x) => pc_fr_into_blst_fr(x),
        }
    }

    fn from_u64(val: u64) -> Self {
        let fr = ArkFr::from(val);
        pc_fr_into_blst_fr(fr)
    }

    fn to_u64_arr(&self) -> [u64; 4] {
        let b = ArkFr::into_repr(&blst_fr_into_pc_fr(self));
        b.0
    }

    fn div(&self, b: &Self) -> Result<Self, String> {
        let a = blst_fr_into_pc_fr(self);
        let b = blst_fr_into_pc_fr(b);
        let div = a / b;
        if div.0 .0.is_empty() {
            Ok(FsFr::zero())
        } else {
            Ok(pc_fr_into_blst_fr(div))
        }
    }

    fn is_one(&self) -> bool {
        blst_fr_into_pc_fr(self).is_one()
    }

    fn is_null(&self) -> bool {
        self.equals(&FsFr(blst_fr {
            l: [
                14526898868952669296,
                2784871451429007392,
                11493358522590675359,
                7138715389977065193,
            ],
        }))
    }

    fn is_zero(&self) -> bool {
        blst_fr_into_pc_fr(self).is_zero()
    }

    fn sqr(&self) -> Self {
        let temp = blst_fr_into_pc_fr(self);
        pc_fr_into_blst_fr(temp.square())
    }

    fn pow(&self, n: usize) -> Self {
        pc_fr_into_blst_fr(blst_fr_into_pc_fr(self).pow([n as u64]))
    }

    fn mul(&self, b: &Self) -> Self {
        pc_fr_into_blst_fr(blst_fr_into_pc_fr(self) * blst_fr_into_pc_fr(b))
    }

    fn add(&self, b: &Self) -> Self {
        pc_fr_into_blst_fr(blst_fr_into_pc_fr(self) + blst_fr_into_pc_fr(b))
    }

    fn sub(&self, b: &Self) -> Self {
        pc_fr_into_blst_fr(blst_fr_into_pc_fr(self) - blst_fr_into_pc_fr(b))
    }

    fn eucl_inverse(&self) -> Self {
        // let mut ret = Self::default();
        // unsafe {
        //     blst_fr_eucl_inverse(&mut ret.0, &self.0);
        // }

        // return ret;
        todo!()
    }

    fn negate(&self) -> Self {
        pc_fr_into_blst_fr(blst_fr_into_pc_fr(self).neg())
    }

    fn inverse(&self) -> Self {
        pc_fr_into_blst_fr(blst_fr_into_pc_fr(self).inverse().unwrap())
    }

    fn equals(&self, b: &Self) -> bool {
        blst_fr_into_pc_fr(self) == blst_fr_into_pc_fr(b)
    }
}

impl Clone for FsFr {
    fn clone(&self) -> Self {
        FsFr(self.0)
    }
}

impl Copy for FsFr {}

pub const SCALE_FACTOR: u64 = 5;
pub const NUM_ROOTS: usize = 32;

impl Poly<FsFr> for LPoly {
    fn default() -> Self {
        Self::new(1).unwrap()
    }

    fn new(size: usize) -> Result<Self, String> {
        Ok(Self {
            coeffs: vec![FsFr::default(); size],
        })
    }

    fn get_coeff_at(&self, i: usize) -> FsFr {
        self.coeffs[i]
    }

    fn set_coeff_at(&mut self, i: usize, x: &FsFr) {
        self.coeffs[i] = *x
    }

    fn get_coeffs(&self) -> &[FsFr] {
        &self.coeffs
    }

    fn len(&self) -> usize {
        self.coeffs.len()
    }

    fn eval(&self, x: &FsFr) -> FsFr {
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

    fn mul_direct(&mut self, x: &Self, len: usize) -> Result<Self, String> {
        poly_mul_direct(self, x, len)
    }

    fn fast_div(&mut self, x: &Self) -> Result<Self, String> {
        poly_fast_div(self, x)
    }
}

impl Clone for LPoly {
    fn clone(&self) -> Self {
        LPoly {
            coeffs: self.coeffs.clone(),
        }
    }
}

impl FFTSettingsPoly<FsFr, LPoly, LFFTSettings> for LFFTSettings {
    fn poly_mul_fft(
        a: &LPoly,
        x: &LPoly,
        len: usize,
        fs: Option<&LFFTSettings>,
    ) -> Result<LPoly, String> {
        poly_mul_fft(a, x, fs, len)
    }
}

impl FFTSettings<FsFr> for LFFTSettings {
    fn new(scale: usize) -> Result<LFFTSettings, String> {
        if scale >= SCALE2_ROOT_OF_UNITY.len() {
            return Err(String::from(
                "Scale is expected to be within root of unity matrix row size",
            ));
        }
        let max_width: usize = 1 << scale;
        let domain = Radix2EvaluationDomain::<ArkFr>::new(max_width as usize).unwrap();

        let roots =
            expand_root_of_unity(&pc_fr_into_blst_fr(domain.group_gen), domain.size as usize)
                .unwrap();

        let mut reverse = roots.clone();
        reverse.reverse();

        Ok(LFFTSettings {
            max_width,
            root_of_unity: pc_fr_into_blst_fr(domain.group_gen),
            expanded_roots_of_unity: roots,
            reverse_roots_of_unity: reverse,
            domain,
        })
    }

    fn get_max_width(&self) -> usize {
        self.max_width
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> FsFr {
        self.expanded_roots_of_unity[i]
    }

    fn get_expanded_roots_of_unity(&self) -> &[FsFr] {
        self.expanded_roots_of_unity.as_slice()
    }

    fn get_reverse_roots_of_unity_at(&self, i: usize) -> FsFr {
        self.reverse_roots_of_unity[i]
    }

    fn get_reversed_roots_of_unity(&self) -> &[FsFr] {
        self.reverse_roots_of_unity.as_slice()
    }
    fn default() -> Self {
        LFFTSettings {
            max_width: 0,
            root_of_unity: FsFr::zero(),
            expanded_roots_of_unity: Vec::new(),
            reverse_roots_of_unity: Vec::new(),
            domain: Radix2EvaluationDomain::<ArkFr>::new(0_usize).unwrap(),
        }
    }
}

impl Clone for LFFTSettings {
    fn clone(&self) -> Self {
        let mut output = LFFTSettings::new(0).unwrap();
        output.max_width = self.max_width;
        output.root_of_unity = self.root_of_unity;
        output.expanded_roots_of_unity = self.expanded_roots_of_unity.clone();
        output.reverse_roots_of_unity = self.reverse_roots_of_unity.clone();
        output
    }
}

impl KZGSettings<FsFr, ArkG1, ArkG2, LFFTSettings, LPoly> for LKZGSettings {
    fn new(
        secret_g1: &[ArkG1],
        secret_g2: &[ArkG2],
        length: usize,
        fs: &LFFTSettings,
    ) -> Result<LKZGSettings, String> {
        Ok(new_kzg_settings(secret_g1, secret_g2, length as u64, fs))
    }

    fn commit_to_poly(&self, p: &LPoly) -> Result<ArkG1, String> {
        Ok(commit(p, self).unwrap())
    }

    fn compute_proof_single(&self, p: &LPoly, x: &FsFr) -> Result<ArkG1, String> {
        Ok(compute_single(p, x, self))
    }

    fn check_proof_single(
        &self,
        com: &ArkG1,
        proof: &ArkG1,
        x: &FsFr,
        value: &FsFr,
    ) -> Result<bool, String> {
        Ok(check_single(com, proof, x, value, self))
    }

    fn compute_proof_multi(&self, p: &LPoly, x: &FsFr, n: usize) -> Result<ArkG1, String> {
        Ok(compute_multi(p, x, n, self))
    }

    fn check_proof_multi(
        &self,
        com: &ArkG1,
        proof: &ArkG1,
        x: &FsFr,
        values: &[FsFr],
        n: usize,
    ) -> Result<bool, String> {
        Ok(check_multi(com, proof, x, values, n, self))
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> FsFr {
        self.fs.get_expanded_roots_of_unity_at(i)
    }

    fn default() -> Self {
        default_kzg()
    }
}

impl Clone for LKZGSettings {
    fn clone(&self) -> Self {
        LKZGSettings::new(
            &self.secret_g1.clone(),
            &self.secret_g2.clone(),
            self.length as usize,
            &self.fs.clone(),
        )
        .unwrap()
    }
}

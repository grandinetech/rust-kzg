use crate::fft::SCALE2_ROOT_OF_UNITY;
use crate::kzg_proofs::{
    check_proof_single as check_single, commit_to_poly as commit,
    compute_proof_single as compute_single, default_kzg, eval_poly, expand_root_of_unity, compute_proof_multi as compute_multi,
    new_kzg_settings, FFTSettings as LFFTSettings, KZGSettings as LKZGSettings, check_proof_multi as check_multi
};
use crate::poly::{poly_inverse, poly_fast_div, poly_mul_direct, poly_long_div, poly_mul_fft};
use crate::utils::PolyData as LPoly;
use ark_bls12_381::{Fr as ArkFr};
use ark_ff::Field;
use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};
use std::ops::Neg;
use ark_ec::models::short_weierstrass_jacobian::GroupProjective;
use ark_std::UniformRand;
use crate::utils::{blst_fr_into_pc_fr, pc_fr_into_blst_fr, pc_g1projective_into_blst_p1};
use blst::{
    blst_fr, blst_fr_eucl_inverse, blst_fr_from_uint64, blst_fr_inverse, blst_fr_sqr, blst_p1, 
    blst_p1_add_or_double, blst_uint64_from_fr
};
// use kzg::DAS;
use kzg::{FFTSettings, Fr, KZGSettings, Poly, G1, G2, FFTSettingsPoly, G1Mul, G2Mul};

#[derive(Debug, PartialEq)]
pub struct ArkG1(pub blst::blst_p1);

impl Clone for ArkG1 {
    fn clone(&self) -> Self {
        ArkG1(self.0.clone())
    }
}

impl G1 for ArkG1 {
    fn default() -> Self {
        Self(blst_p1::default())
    }

    fn add_or_dbl(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_p1_add_or_double(&mut ret.0, &self.0, &b.0);
        }
        ret
    }

    fn equals(&self, b: &Self) -> bool {
        self.0.eq(&b.0)
    }

    // fn zero() -> ArkG1 {
    //     ArkG1(blst_p1 {
    //         x: blst_fp {
    //             l: [0, 0, 0, 0, 0, 0],
    //         },
    //         y: blst_fp {
    //             l: [0, 0, 0, 0, 0, 0],
    //         },
    //         z: blst_fp {
    //             l: [0, 0, 0, 0, 0, 0],
    //         },
    //     })
    // }

    fn rand() -> Self {
        let mut rng = ark_std::test_rng();
        pc_g1projective_into_blst_p1(GroupProjective::rand(&mut rng)).unwrap()
    }

    fn destroy(&mut self) {}

    fn identity() -> Self {
        todo!()
    }

    fn generator() -> Self {
        todo!()
    }

    fn negative_generator() -> Self {
        todo!()
    }

    fn is_inf(&self) -> bool {
        todo!()
    }

    fn dbl(&self) -> Self {
        todo!()
    }

    fn sub(&self, b: &Self) -> Self {
        todo!()
    }
}

impl G1Mul<FsFr> for ArkG1 {
    fn mul(&self, b: &FsFr) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub struct ArkG2(pub blst::blst_p2);

impl Clone for ArkG2 {
    fn clone(&self) -> Self {
        ArkG2(self.0.clone())
    }
}

impl G2 for ArkG2 {
    fn default() -> Self {
        todo!()
    }

    fn generator() -> Self {
        todo!()
    }

    fn negative_generator() -> Self {
        todo!()
    }

    fn add_or_dbl(&self, b: &Self) -> Self {
        todo!()
    }

    fn dbl(&self) -> Self {
        todo!()
    }

    fn sub(&self, b: &Self) -> Self {
        todo!()
    }

    fn equals(&self, b: &Self) -> bool {
        todo!()
    }

    fn destroy(&mut self) {
        todo!()
    }
}

impl G2Mul<FsFr> for ArkG1 {
    fn mul(&self, b: &FsFr) -> Self {
        todo!()
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

    fn one() -> Self {
        Self::from_u64(1)
    }

    fn rand() -> Self {
        let val: [u64; 4] = rand::random();
        let mut ret = Self::default();
        unsafe {
            blst_fr_from_uint64(&mut ret.0, val.as_ptr());
        }

        ret
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

	fn to_u64_arr(&self) -> [u64; 4] {
		todo!()
	}
	
	fn div(&self, b: &Self) -> Result<Self, String>{
		todo!()
	}
	
    fn is_one(&self) -> bool {
        let mut val: [u64; 4] = [0; 4];
        unsafe {
            blst_uint64_from_fr(val.as_mut_ptr(), &self.0);
        }
        return val[0] == 1 && val[1] == 0 && val[2] == 0 && val[3] == 0;
    }

    fn is_zero(&self) -> bool {
        let mut val: [u64; 4] = [0; 4];
        unsafe {
            blst_uint64_from_fr(val.as_mut_ptr(), &self.0);
        }
        return val[0] == 0 && val[1] == 0 && val[2] == 0 && val[3] == 0;
        // self.0.l[0] == 0 && self.0.l[1] == 0 && self.0.l[2] == 0 && self.0.l[3] == 0
    }

    fn sqr(&self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_sqr(&mut ret.0, &self.0);
        }

        ret
    }

    fn pow(&self, n: usize) -> Self {
        pc_fr_into_blst_fr(blst_fr_into_pc_fr(self).pow(&[n as u64]))
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
        let mut ret = Self::default();
        unsafe {
            blst_fr_eucl_inverse(&mut ret.0, &self.0);
        }

        return ret;
    }

    fn negate(&self) -> Self {
        pc_fr_into_blst_fr(blst_fr_into_pc_fr(self).neg())
    }

    fn inverse(&self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_inverse(&mut ret.0, &self.0);
        }

        ret
    }

    fn equals(&self, b: &Self) -> bool {
        let mut val_a: [u64; 4] = [0; 4];
        let mut val_b: [u64; 4] = [0; 4];

        unsafe {
            blst_uint64_from_fr(val_a.as_mut_ptr(), &self.0);
            blst_uint64_from_fr(val_b.as_mut_ptr(), &b.0);
        }

        return val_a[0] == val_b[0]
            && val_a[1] == val_b[1]
            && val_a[2] == val_b[2]
            && val_a[3] == val_b[3];
    }

    fn destroy(&mut self) {}
}

impl Clone for FsFr {
    fn clone(&self) -> Self {
        FsFr(self.0.clone())
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
        self.coeffs[i] = x.clone()
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
        let scale_factor = FsFr::from_u64(SCALE_FACTOR);
        let inv_factor = scale_factor.inverse();

        let mut factor_power = FsFr::one();
        for i in 0..self.coeffs.len() {
            factor_power = factor_power.mul(&inv_factor);
            self.coeffs[i] = self.coeffs[i].mul(&factor_power);
        }
    }

    fn unscale(&mut self) {
        let scale_factor = FsFr::from_u64(SCALE_FACTOR);

        let mut factor_power = FsFr::one();
        for i in 0..self.coeffs.len() {
            factor_power = factor_power.mul(&scale_factor);
            self.coeffs[i] = self.coeffs[i].mul(&factor_power);
        }
    }

    fn inverse(&mut self, new_len: usize) -> Result<Self, String> {
        poly_inverse(self, new_len)
    }

    fn div(&mut self, x: &Self) -> Result<Self, String> {
        // pc_poly_into_blst_poly(
        //     &blst_poly_into_pc_poly(self).unwrap() / &blst_poly_into_pc_poly(x).unwrap(),
        // )
        Ok(poly_fast_div(self, x).unwrap())
    }

    fn long_div(&mut self, x: &Self) -> Result<Self, String> {
        Ok(poly_long_div(self, x))
    }

    fn mul_direct(&mut self, x: &Self, len: usize) -> Result<Self, String> {
        poly_mul_direct(self, x, len)
    }

    fn fast_div(&mut self, x: &Self) -> Result<Self, String>  {
        Ok(poly_fast_div(self, x).unwrap())
    }

    fn destroy(&mut self) {}
}

impl Clone for LPoly {
    fn clone(&self) -> Self {
        LPoly {
            coeffs: self.coeffs.clone(),
        }
    }
}

impl FFTSettingsPoly<FsFr, LPoly, LFFTSettings> for LFFTSettings {
    fn poly_mul_fft(a: &LPoly, x: &LPoly, len: usize, fs: Option<&LFFTSettings>) -> Result<LPoly, String> {
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
            max_width: max_width,
            root_of_unity: pc_fr_into_blst_fr(domain.group_gen),
            expanded_roots_of_unity: roots,
            reverse_roots_of_unity: reverse,
            domain: domain
        })
    }

    fn get_max_width(&self) -> usize {
        self.max_width
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> FsFr {
        self.expanded_roots_of_unity[i]
    }

    fn get_expanded_roots_of_unity(&self) -> &[FsFr] {
        &self.expanded_roots_of_unity.as_slice()
    }

    fn get_reverse_roots_of_unity_at(&self, i: usize) -> FsFr {
        self.reverse_roots_of_unity[i]
    }

    fn get_reversed_roots_of_unity(&self) -> &[FsFr] {
        &self.reverse_roots_of_unity.as_slice()
    }
    fn default() -> Self {
        LFFTSettings {
            max_width: 0,
            root_of_unity: FsFr::zero(),
            expanded_roots_of_unity: Vec::new(),
            reverse_roots_of_unity: Vec::new(),
            domain: Radix2EvaluationDomain::<ArkFr>::new(0 as usize).unwrap(),
        }
    }
    fn destroy(&mut self) {}
}

impl Clone for LFFTSettings {
    fn clone(&self) -> Self {
        let mut output = LFFTSettings::new(0).unwrap();
        output.max_width = self.max_width;
        output.root_of_unity = self.root_of_unity.clone();
        output.expanded_roots_of_unity = self.expanded_roots_of_unity.clone();
        output.reverse_roots_of_unity = self.reverse_roots_of_unity.clone();
        output
    }
}

impl KZGSettings<FsFr, ArkG1, ArkG2, LFFTSettings, LPoly> for LKZGSettings {
    fn new(
        secret_g1: &Vec<ArkG1>,
        secret_g2: &Vec<ArkG2>,
        length: usize,
        fs: LFFTSettings,
    ) -> Result<LKZGSettings, String> {
        Ok(new_kzg_settings(secret_g1, secret_g2, length as u64, fs))
    }

    fn commit_to_poly(&self, p: &LPoly) -> Result<ArkG1, String> {
        Ok(commit(p, self).unwrap())
    }

    fn compute_proof_single(&self, p: &LPoly, x: &FsFr) -> Result<ArkG1, String> {
        Ok(compute_single(p, x, self))
    }

    fn check_proof_single(&self, com: &ArkG1, proof: &ArkG1, x: &FsFr, value: &FsFr) -> Result<bool, String> {
        Ok(check_single(com, proof, x, value, self))
    }

    fn compute_proof_multi(&self, p: &LPoly, x: &FsFr, n: usize) -> Result<ArkG1, String> {
        Ok(compute_multi(p, x, n, self))
    }

    fn check_proof_multi(&self, com: &ArkG1, proof: &ArkG1, x: &FsFr, values: &Vec<FsFr>, n: usize) -> Result<bool, String> {
        Ok(check_multi(com, proof, x, values, n, self))
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> FsFr {
        self.fs.get_expanded_roots_of_unity_at(i)
    }

    fn default() -> Self {
        default_kzg()
    }

    fn destroy(&mut self) {}
}

impl Clone for LKZGSettings {
    fn clone(&self) -> Self {
        LKZGSettings::new(&self.secret_g1.clone(), &self.secret_g2.clone(), self.length as usize, self.fs.clone()).unwrap()
    }
}